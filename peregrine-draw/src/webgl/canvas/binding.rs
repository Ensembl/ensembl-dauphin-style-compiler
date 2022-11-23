use std::{collections::{BTreeMap}, sync::{Arc, Mutex}};
use peregrine_toolkit::{lock, error::Error};
use web_sys::{WebGlTexture, WebGlRenderingContext};
use crate::webgl::CanvasWeave;

trait TextureProfile {
    fn create(&self, context: &WebGlRenderingContext, weave: &CanvasWeave) -> WebGlTexture;
    fn destroy(&self, context: &WebGlRenderingContext, texture: &WebGlTexture);
}

struct BoundSlotState {
    br: BoundRef,
    texture: WebGlTexture
}

#[derive(Clone)]
pub(crate) struct SlotToken {
    weave: CanvasWeave,
    binding: Arc<Mutex<BindingState>>,
    bound: Arc<Mutex<Option<BoundSlotState>>>
}

impl SlotToken {
    fn activate(&self, context: &WebGlRenderingContext) -> Result<(WebGlTexture,u32),Error> {
        lock!(self.binding).activate(context,&self.weave,&self.bound)?;
        let state = lock!(self.bound);
        let state = state.as_ref().expect("missing bound data");
        Ok((state.texture.clone(),state.br.slot as u32))
    }
}

impl Drop for SlotToken {
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

struct TokenStateStore(Vec<Option<Arc<Mutex<Option<BoundSlotState>>>>>);

impl TokenStateStore {
    fn set(&mut self, slot: usize, state: &Arc<Mutex<Option<BoundSlotState>>>) {
        if self.0.len() <= slot {
            self.0.resize(slot+1,None);
        }
        self.0[slot] = Some(state.clone());
    }

    fn get(&self, slot: usize) -> Result<&Arc<Mutex<Option<BoundSlotState>>>,Error> {
        self.0.get(slot)
            .map(|x| x.as_ref())
            .flatten()
            .ok_or_else(|| Error::fatal("getting unoccupied slot"))
    }

    fn clear(&mut self, slot: usize) {
        if let Some(entry) = self.0.get_mut(slot) {
            *entry = None;
        }
    }
}
/*
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
 * ACTIVE or VESTIGIAL are known together as BOUND and have a BoundRef.
 */

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

struct BindingState {
    profile: Box<dyn TextureProfile>,
    slot_source: SlotSource,
    state_machine: StateMachine,
    token_state: TokenStateStore,
    dead_textures: Vec<WebGlTexture>
}

impl BindingState {
    fn new(profile: Box<dyn TextureProfile>, max_slots: usize) -> BindingState {
        BindingState {
            profile,
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
     fn drop_slot(&mut self, br: BoundRef, texture: WebGlTexture) {
        self.token_state.clear(br.slot);  /* remove ref to our state */
        self.slot_source.free(br.slot);   /* slot can be reused */
        self.state_machine.free(br);      /* remove from state machine */
        self.dead_textures.push(texture); /* texture can be tidied */
    }

    fn get_slot(&mut self, context: &WebGlRenderingContext) -> Result<usize,Error> {
        if let Some(slot) = self.slot_source.allocate() {
            return Ok(slot);
        }
        let slot = self.state_machine.steal()?;
        if let Some(victim) = lock!(self.token_state.get(slot)?).as_ref() {
            self.profile.destroy(context,&victim.texture);
        }
        self.token_state.clear(slot);
        Ok(slot)
    }

    fn allocate(&mut self, context: &WebGlRenderingContext, weave: &CanvasWeave, binding: &Arc<Mutex<BindingState>>) -> Result<SlotToken,Error> {
        self.tidy(context);
        Ok(SlotToken {
            binding: binding.clone(),
            weave: weave.clone(),
            bound: Arc::new(Mutex::new(None))
        })
    }

    fn activate(&mut self, context: &WebGlRenderingContext, weave: &CanvasWeave, bound: &Arc<Mutex<Option<BoundSlotState>>>) -> Result<(),Error> {
        self.tidy(context);
        let mut bound_state = lock!(bound);
        if let Some(bound) = bound_state.as_mut() {
            /* vestigial or active */
            self.state_machine.activate(&mut bound.br);
            return Ok(());
        }
        /* unbound */
        drop(bound_state);
        let texture = self.profile.create(context,weave);
        let slot = self.get_slot(context)?;
        let mut br = self.state_machine.create(slot); /* unbound -> vestigial */
        self.state_machine.activate(&mut br);                /* vestigial -> active */
        *lock!(bound) = Some(BoundSlotState { texture, br });
        self.token_state.set(slot,bound);
        self.tidy(context);
        Ok(())
    }

    fn clear(&mut self, context: &WebGlRenderingContext) {
        self.tidy(context);
        self.state_machine.clear();
    }

    fn tidy(&mut self, context: &WebGlRenderingContext) {
        for texture in self.dead_textures.drain(..) {
            self.profile.destroy(context,&texture);
        }
    }
}

#[derive(Clone)]
pub(crate) struct Binding(Arc<Mutex<BindingState>>);

impl Binding {
    fn new<F>(profile: F, max_slots: usize) -> Binding where F: TextureProfile + 'static {
        Binding(Arc::new(Mutex::new(BindingState::new(Box::new(profile),max_slots))))
    }

    fn new_token(&self, context: &WebGlRenderingContext, weave: &CanvasWeave) -> Result<SlotToken,Error> {
        let binding = self.clone();
        lock!(self.0).allocate(context,weave,&binding.0)
    }

    fn clear(&self, context: &WebGlRenderingContext) {
        lock!(self.0).clear(context);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_smoke() {

    }
}
