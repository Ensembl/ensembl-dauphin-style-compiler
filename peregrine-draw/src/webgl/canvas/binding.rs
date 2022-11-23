use std::{collections::{BTreeMap}, sync::{Arc, Mutex}, marker::PhantomData};
use peregrine_toolkit::{lock, error::Error};

/* Manages the abstract part of managing texture bindings. Is polymorphic on anything webgl-
 * specific C => WebGlContext, W => CanvasWeave, X => WebGlTexture. This serves to make the
 * algorithm testable as a unit and to decouple the code. A concrete implementation must provide
 * an implementation of TextureProfile which does the texture allocation and destroying.
 * 
 * The basic setup is of a cache which lazily leaves textures "hanging around" to the best of
 * its ability, to reduce the number of creates/destroys required. The general algorithm for
 * using textures is that from a blank state textures are added as needed. That "as needed" step
 * may need to evict currently unneeded textures to avoid exceeding GPU limits. It must never
 * evict known-needed textures for that run. Once run these textures are not destroyed, but kept
 * around to be hopefully useful next time. Externally the operations are
 * 
 * clear -- we are about to start a run: forget about neededness of all canaves
 * activate -- this canvas is needed for this run: give it a slot
 * free -- this canvas has been dropped, it is no longer active and may be reused.
 * 
 * To allow on-drop semantics, we hand out a token, `SlotToken` which a canvas keeps around. This
 * does two things. First, it provides an activate method. Second, when dropped it calls free.
 * 
 * Slot tokens are allocated from a `Binding`, which also provides clear(). All other structures
 * in here are internal-only.
 *
 * 
 * Internally, there are two major structs maintaining state:
 * 1. BoundSlotState, of which there is one per allocated `SlotToken`;
 * 2. BindingState, which is global across the whole `Binding`.
 * 
 * All major operations happen in the context of BindingState. Both Binding and every SlotToken
 * have Arc+Mutex references to BindingState. Deadlock is avoided by calls from Binding never
 * calling methods in SlotToken and vice-versa. Both may then freely lock BindingState without
 * worrying that there's a lock out on it already. There is an extra complication with the dropper
 * of SlotToken, which locks BindingState. To prevent this, no SlotToken is permitted to be handled
 * inside BindingState. This ensures that the dropper cannot run while it is locked. BOundSlotState
 * is only ever kept locked momentarily and so doesn't cause issues.
 * 
 * The SlotToken contains the Arc+Mutex to the two state data-structures. Its internal state is the
 * current texture and a BoundRef. A BoundRef is a handle used to identify itself to BindingState
 * for delegated operations.
 * 
 * A BoundRef contains two values, a "timestamp" and a slot. The timestamp is a global integer which
 * increases whenever any texture is activated, any does double-duty as a means of identifying
 * SlotTokens and as a way of implementing LRU. It is reasonably compact, but sometimes double-ticks
 * for implementation-convenience so don't worry about missing values: it's there for ordering.
 * 
 * BindingState contains a number of sub-structures, managed individually for cleanliness.
 *  
 * token_state -- has copies of the BoundSlotState for every currently bound slot (active or just
 *     cached), organised by slot. This allows Binding to change the state of tokens when it needs
 *     to steal one.
 * 
 * slot_source -- manages any unused slots either on startup or after freeing.
 * 
 * state_machine -- a per-token state machine:
 *     active -- bound and in use
 *     vestigial -- bound but not yet in use
 *     unbound -- not bound
 * 
 * dead_textures -- we don't want to be doing WebGL operations in destructors so when a slot is
 *     dropped the texture is moved into this queue. On the next operation they are tidied away.
 * 
 * The state machine is as follows, with transitions for the operations described above. create
 * is the start of the activation process when a free slot is found. When a slot must be stolem,
 * the three-phase process is steal (victim); steal (thief); and finally activate. Where activation
 * benefits from a vestigial texture, just the activate step is run.
 * 
 *               steal (victim)
 *    +---------------------------------------------------+
 *    |                                                   |
 *    |          free                                     |
 *    |  +----------------------------------------------+ |
 *    |  |                                              | |
 *    v  v       free                    clear          | |
 *   [UNBOUND] <------------ [ACTIVE] -----------> [VESTIGIAL]
 *        |                        ^                   |  ^
 *        |                        +-------------------+  |
 *        |                              activate         |
 *        |         create OR steal (thief)               |
 *        +-----------------------------------------------+
 * 
 * The state machine maintains state as two B-tree maps, one for vestigial tokens and one for
 * active. Tokens are never in both of these. Tokens in neither are unbound. Both trees map from
 * timestamp to slot. The active B-Tree is transferred to the vestigial tree in its entiriy on
 * clear, and removed piece-wise upon freeing. The vesitigal B-Tree is removed piece-wise following
 * activation or freeing. For stealing, the smallest key is removed for timestamping. The timestamp
 * must be updated during thr activate step. The only two transitions into vestigial or active
 * states are this transition and the create/steal(thief) states which are always part of
 * the activation step. If the timestamp is updated on both transitions, it can be used as a unique
 * key for these trees.
 */

#[derive(Debug)]
pub(crate) struct Stats {
    pub activations: u64,
    pub creates: u64
}

impl Stats {
    fn new() -> Stats { Stats { activations: 0, creates: 0 } }
    fn activate(&mut self) { self.activations += 1; }
    fn create(&mut self) { self.creates += 1; }
}

pub(crate) trait TextureProfile<C,W,X,E> {
    fn create(&mut self, context: &C, weave: W, slot: usize) -> Result<X,E>;
    fn destroy(&mut self, context: &C, texture: &X);
    fn no_slots(&self) -> E;
    fn stats(&mut self, _stats: &Stats) {}
}

struct BoundSlotState<X>{
    br: BoundRef,
    texture: X
}

#[derive(Clone)]
pub(crate) struct SlotToken<C,W,X,E> {
    weave: PhantomData<W>,
    binding: Arc<Mutex<BindingState<C,W,X,E>>>,
    bound: Arc<Mutex<Option<BoundSlotState<X>>>>
}

impl<C,W,X,E> SlotToken<C,W,X,E> where X: Clone {
    pub(crate) fn activate(&self, weave: W, context: &C) -> Result<(X,u32),E> {
        lock!(self.binding).activate(context,weave,&self.bound)?;
        let state = lock!(self.bound);
        let state = state.as_ref().expect("missing bound data");
        Ok((state.texture.clone(),state.br.slot as u32))
    }
}

impl<C,W,X,E> Drop for SlotToken<C,W,X,E> {
    fn drop(&mut self) {
        /* free() */
        let state = lock!(self.bound).take();
        if let Some(state) = state {
            lock!(self.binding).drop_slot(state.br,state.texture);
        }
    }
}

struct SlotSource {
    next_free_slot: usize,
    max_slots: usize,
    unused_slot: Vec<usize>
}

impl SlotSource {
    fn allocate(&mut self) -> Option<usize> {
        if self.next_free_slot < self.max_slots {
            self.next_free_slot += 1;
            Some(self.next_free_slot - 1)
        } else if let Some(slot) = self.unused_slot.pop() {
            Some(slot)
        } else {
            None
        }
    }

    fn free(&mut self, slot: usize) {
        self.unused_slot.push(slot);
    }
}

struct TokenStateStore<X>(Vec<Option<Arc<Mutex<Option<BoundSlotState<X>>>>>>);

impl<X> TokenStateStore<X> {
    fn set(&mut self, slot: usize, state: &Arc<Mutex<Option<BoundSlotState<X>>>>) {
        if self.0.len() <= slot {
            self.0.resize(slot+1,None);
        }
        self.0[slot] = Some(state.clone());
    }

    fn get(&self, slot: usize) -> &Arc<Mutex<Option<BoundSlotState<X>>>> {
        self.0.get(slot)
            .map(|x| x.as_ref())
            .flatten()
            .expect("getting unoccupied slot")
    }

    fn clear(&mut self, slot: usize) {
        if let Some(entry) = self.0.get_mut(slot) {
            *entry = None;
        }
    }
}
struct BoundRef { timestamp: i64, slot: usize }

struct StateMachine {
    next_timestamp: i64,
    vestigial_lru: BTreeMap<i64,usize>, /* timestamp -> slot */
    active: BTreeMap<i64,usize>
}

impl StateMachine {
    fn free(&mut self, br: BoundRef) {
        self.vestigial_lru.remove(&br.timestamp);
        self.active.remove(&br.timestamp);
    }

    fn activate(&mut self, br: &mut BoundRef) {
        let slot = self.vestigial_lru.remove(&br.timestamp)
            .or_else(|| self.active.remove(&br.timestamp));
        if let Some(slot) = slot {
            br.timestamp = self.next_timestamp;
            self.next_timestamp += 1;
            self.active.insert(br.timestamp,slot);
        }
    }

    fn create(&mut self, slot: usize) -> BoundRef {
        let timestamp = self.next_timestamp;
        self.next_timestamp += 1;
        self.vestigial_lru.insert(timestamp,slot);
        BoundRef { timestamp, slot }
    }

    fn clear(&mut self) {
        for (timestamp,slot) in self.active.iter() {
            self.vestigial_lru.insert(*timestamp,*slot);
        }
        self.active.clear();
    }

    fn steal(&mut self) -> Result<usize,Error> {
        if let Some((timestamp,slot)) = self.vestigial_lru.iter().next().map(|(x,y)| (*x,*y)) {
            self.vestigial_lru.remove(&timestamp); /* unbind old */
            Ok(slot)
        } else {
            Err(Error::fatal("no slots available"))
        }
    }
}

struct BindingState<C,W,X,E> {
    profile: Box<dyn TextureProfile<C,W,X,E>>,
    stats: Stats,
    slot_source: SlotSource,
    state_machine: StateMachine,
    token_state: TokenStateStore<X>,
    dead_textures: Vec<X>
}

impl<C,W,X,E> BindingState<C,W,X,E> {
    fn new(profile: Box<dyn TextureProfile<C,W,X,E>>, max_slots: usize) -> BindingState<C,W,X,E> {
        BindingState {
            profile,
            stats: Stats::new(),
            slot_source: SlotSource { next_free_slot: 0, max_slots, unused_slot: vec![] },
            token_state: TokenStateStore(vec![]),
            state_machine: StateMachine {
                next_timestamp: 0,
                vestigial_lru: BTreeMap::new(),
                active: BTreeMap::new()
            },
            dead_textures: vec![]
        }
    }

    /* Drop on Active or Vestigial slot due to individual token being dropped.
     */
     fn drop_slot(&mut self, br: BoundRef, texture: X) {
        self.token_state.clear(br.slot);  /* remove ref to our state */
        self.slot_source.free(br.slot);   /* slot can be reused */
        self.state_machine.free(br);      /* remove from state machine */
        self.dead_textures.push(texture); /* texture can be tidied */
    }

    fn get_slot(&mut self, context: &C) -> Result<usize,Error> {
        if let Some(slot) = self.slot_source.allocate() {
            return Ok(slot);
        }
        let slot = self.state_machine.steal()?;
        if let Some(victim) = lock!(self.token_state.get(slot)).as_ref() {
            self.profile.destroy(context,&victim.texture);
        }
        self.token_state.clear(slot);
        Ok(slot)
    }

    fn activate(&mut self, context: &C, weave: W, bound: &Arc<Mutex<Option<BoundSlotState<X>>>>) -> Result<(),E> {
        self.tidy(context);
        let mut bound_state = lock!(bound);
        if let Some(bound) = bound_state.as_mut() {
            /* vestigial or active */
            self.state_machine.activate(&mut bound.br);
            self.stats.activate();
            self.profile.stats(&self.stats);
            return Ok(());
        }
        /* unbound */
        drop(bound_state);
        let slot = self.get_slot(context).map_err(|e| self.profile.no_slots())?;
        let texture = self.profile.create(context,weave,slot)?;
        self.stats.activate();
        self.stats.create();
        let mut br = self.state_machine.create(slot); /* unbound -> vestigial */
        self.state_machine.activate(&mut br);                /* vestigial -> active */
        *lock!(bound) = Some(BoundSlotState { texture, br });
        self.token_state.set(slot,bound);
        self.tidy(context);
        self.profile.stats(&self.stats);
        Ok(())
    }

    fn clear(&mut self, context: &C) {
        self.tidy(context);
        self.state_machine.clear();
    }

    fn tidy(&mut self, context: &C) {
        for texture in self.dead_textures.drain(..) {
            self.profile.destroy(context,&texture);
        }
    }
}

#[derive(Clone)]
pub(crate) struct Binding<C,W,X,E>(Arc<Mutex<BindingState<C,W,X,E>>>);

impl<C,W,X,E> Binding<C,W,X,E> {
    pub(crate) fn new<F>(profile: F, max_slots: usize) -> Binding<C,W,X,E> where F: TextureProfile<C,W,X,E> + 'static {
        Binding(Arc::new(Mutex::new(BindingState::new(Box::new(profile),max_slots))))
    }

    pub(crate) fn new_token(&self, context: &C) -> Result<SlotToken<C,W,X,E>,Error> {
        lock!(self.0).tidy(context);
        Ok(SlotToken {
            binding: self.0.clone(),
            weave: PhantomData,
            bound: Arc::new(Mutex::new(None))
        })
    }

    pub(crate) fn clear(&self, context: &C) {
        lock!(self.0).clear(context);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Clone)]
    struct BindingProfile(Arc<Mutex<(Vec<(usize,usize,Option<usize>)>,(u64,u64))>>);

    impl TextureProfile<usize,usize,usize,usize> for BindingProfile {
        fn create(&mut self, context: &usize, weave: usize, slot: usize) -> Result<usize,usize> {
            lock!(self.0).0.push((*context,weave,Some(slot)));
            if weave > 50000 { return Err(weave); }
            Ok(context+weave)
        }

        fn no_slots(&self) -> usize { 42 }

        fn destroy(&mut self, context: &usize, texture: &usize) {
            lock!(self.0).0.push((*context,*texture,None));        
        }

        fn stats(&mut self, stats: &Stats) {
            lock!(self.0).1 = (stats.activations,stats.creates);
        }
    }

    fn check_active(binding: &Binding<usize,usize,usize,usize>, cmp: &[(i64,usize)]) {
        let got = lock!(binding.0).state_machine.active.iter()
            .map(|(a,b)| (*a,*b))
            .collect::<Vec<_>>();
        assert_eq!(got,cmp);
    }

    fn check_vestigial(binding: &Binding<usize,usize,usize,usize>, cmp: &[(i64,usize)]) {
        let got = lock!(binding.0).state_machine.vestigial_lru.iter()
            .map(|(a,b)| (*a,*b))
            .collect::<Vec<_>>();
        assert_eq!(got,cmp);
    }

    /* None -- No entry for this slot (unbound)
     * Some(None) -- Should be impossible: during activation, the only place to set the token_state,
     *               the BoundSlotState is initialised and is never cleared.
     * Some(Some((timestamp,slot,texture)))
     */
    fn check_token_state(binding: &Binding<usize,usize,usize,usize>, cmp: &[Option<Option<(i64,usize,usize)>>]) {
        let state = lock!(binding.0).token_state.0.iter().map(|x| {
            x.as_ref().map(|y| {
                lock!(y).as_ref().map(|z| {
                    (z.br.timestamp,z.br.slot,z.texture)
                })
            })
        }).collect::<Vec<_>>();
        assert_eq!(cmp,state);
    }

    #[test]
    fn test_smoke() {
        let profile = BindingProfile(Arc::new(Mutex::new((vec![],(0,0)))));
        let binding = Binding::new(profile.clone(),4);
        /* create ten tokens, check not much has changed */
        let mut tt = (0..10)
            .map(|i| binding.new_token(&1000) )
            .collect::<Result<Vec<_>,_>>().ok().unwrap();
        assert_eq!(lock!(profile.0).0,vec![]);
        assert_eq!(lock!(profile.0).1,(0,0));
        assert_eq!(lock!(binding.0).dead_textures.len(),0);
        assert_eq!(lock!(binding.0).state_machine.next_timestamp,0);
        assert_eq!(lock!(binding.0).state_machine.vestigial_lru.len(),0);
        assert_eq!(lock!(binding.0).state_machine.active.len(),0);
        assert_eq!(lock!(binding.0).slot_source.next_free_slot,0);
        assert_eq!(lock!(binding.0).slot_source.max_slots,4);
        assert_eq!(lock!(binding.0).slot_source.unused_slot.len(),0);
        assert_eq!(lock!(binding.0).token_state.0.len(),0);
        /* first activation */
        assert_eq!((1000,0),tt[0].activate(0,&1000).ok().expect("A"));
        assert_eq!(lock!(profile.0).0,vec![(1000,0,Some(0))]);
        assert_eq!(lock!(profile.0).1,(1,1));
        assert_eq!(lock!(binding.0).dead_textures.len(),0);
        assert_eq!(lock!(binding.0).state_machine.next_timestamp,2);
        assert_eq!(lock!(binding.0).state_machine.vestigial_lru.len(),0);
        check_active(&binding,&[(1,0)]);
        assert_eq!(lock!(binding.0).slot_source.next_free_slot,1);
        assert_eq!(lock!(binding.0).slot_source.unused_slot.len(),0);
        check_token_state(&binding,&[Some(Some((1,0,1000)))]);
        /* second to fourth activations */
        let mut p = vec![(1000,0,Some(0))];
        let mut act = vec![(1,0)];
        let mut ts = vec![Some(Some((1,0,1000)))];
        for i in 1..4 {
            assert_eq!((1000+100*i,i as u32),tt[i].activate(100*i,&1000).ok().expect("B"));
            p.push((1000,100*i,Some(i)));
            act.push((1+2*i as i64,i));
            ts.push(Some(Some((1+2*i as i64,i,1000+100*i))));
            assert_eq!(*lock!(profile.0).0,p);
            assert_eq!(lock!(profile.0).1,(1+i as u64,1+i as u64));
            assert_eq!(lock!(binding.0).dead_textures.len(),0);
            assert_eq!(lock!(binding.0).state_machine.next_timestamp,2+2*i as i64);
            assert_eq!(lock!(binding.0).state_machine.vestigial_lru.len(),0);
            check_active(&binding,&act);
            assert_eq!(lock!(binding.0).slot_source.next_free_slot,i+1);
            assert_eq!(lock!(binding.0).slot_source.unused_slot.len(),0);
            check_token_state(&binding,&ts);    
        }
        /* fifth activation should fail as we have four active slots */
        assert!(tt[4].activate(400,&1000).is_err());
        /* clear so that everything is available again */
        binding.clear(&1000);
        assert_eq!(*lock!(profile.0).0,p);
        assert_eq!(lock!(profile.0).1,(4,4));
        assert_eq!(lock!(binding.0).dead_textures.len(),0);
        assert_eq!(lock!(binding.0).state_machine.next_timestamp,8);
        check_vestigial(&binding,&[(1,0),(3,1),(5,2),(7,3)]);
        check_active(&binding,&[]);
        assert_eq!(lock!(binding.0).slot_source.next_free_slot,4);
        assert_eq!(lock!(binding.0).slot_source.unused_slot.len(),0);
        check_token_state(&binding,&ts);
        /* fifth activation, reactivate first handle to add some interest to the lru */
        assert_eq!((1000,0),tt[0].activate(0,&1000).ok().expect("C"));
        assert_eq!(*lock!(profile.0).0,p);
        assert_eq!(lock!(profile.0).1,(5,4));
        assert_eq!(lock!(binding.0).dead_textures.len(),0);
        assert_eq!(lock!(binding.0).state_machine.next_timestamp,9);
        check_vestigial(&binding,&[(3,1),(5,2),(7,3)]);
        check_active(&binding,&[(8,0)]);
        assert_eq!(lock!(binding.0).slot_source.unused_slot.len(),0);
        ts[0] = Some(Some((8,0,1000)));
        check_token_state(&binding,&ts);
        /* mess with state some more, drop the second allocation! */
        tt[1] = tt[0].clone();
        assert_eq!(*lock!(profile.0).0,p);
        assert_eq!(lock!(profile.0).1,(5,4));
        assert_eq!(lock!(binding.0).dead_textures.len(),1);
        assert_eq!(lock!(binding.0).state_machine.next_timestamp,9);
        check_vestigial(&binding,&[(5,2),(7,3)]);
        check_active(&binding,&[(8,0)]);
        assert_eq!(lock!(binding.0).slot_source.unused_slot,vec![1]);
        ts[1] = None;
        check_token_state(&binding,&ts);
        /* sixth activation should proceed from free slot without evictions */
        assert_eq!((1500,1),tt[5].activate(500,&1000).ok().expect("D"));
        p.push((1000,1100,None)); // from last change: tidy has now run.
        p.push((1000,500,Some(1))); // this is our one.
        assert_eq!(*lock!(profile.0).0,p);
        assert_eq!(lock!(profile.0).1,(6,5));
        assert_eq!(lock!(binding.0).dead_textures.len(),0);
        assert_eq!(lock!(binding.0).state_machine.next_timestamp,11);
        check_vestigial(&binding,&[(5,2),(7,3)]);
        check_active(&binding,&[(8,0),(10,1)]);
        assert_eq!(lock!(binding.0).slot_source.unused_slot.len(),0);
        ts[1] = Some(Some((10,1,1500)));
        check_token_state(&binding,&ts);
        /* seventh activation should evict/allocate slot 2 */
        assert_eq!((1600,2),tt[6].activate(600,&1000).ok().expect("E"));
        p.push((1000,1200,None)); // slot 2 evicted
        p.push((1000,600,Some(2))); // this is us
        assert_eq!(*lock!(profile.0).0,p);
        assert_eq!(lock!(profile.0).1,(7,6));
        assert_eq!(lock!(binding.0).dead_textures.len(),0);
        assert_eq!(lock!(binding.0).state_machine.next_timestamp,13);
        check_vestigial(&binding,&[(7,3)]);
        check_active(&binding,&[(8,0),(10,1),(12,2)]);
        assert_eq!(lock!(binding.0).slot_source.unused_slot.len(),0);
        ts[2] = Some(Some((12,2,1600)));
        check_token_state(&binding,&ts);
        drop(tt);
    }
}
