/* A party is a means of managing a set of elements T1,T2,... created from some
 * specification, x1, x2, .... A call from outside sets the set membership, and callbacks
 * are used to create and destroy elements as necessary. Elements may take some time to
 * create. It will only be created once while pending. Once done it is placed in the
 * set "proper". This intermediate state may have a different type, if required.
 * 
 * The callbacks are specified by a trait passed at constrcution time. To avoid chaos,
 * transitions out of pending only occur following a ping(), the polling method.
 */

use std::{hash::Hash, collections::{HashMap, HashSet}, mem, iter::FromIterator};
use lazy_static::lazy_static;
use identitynumber::identitynumber;

pub trait PartyActions<X,P,T> {
    /* start creating an object */
    fn ctor(&mut self, index: &X) -> P;

    /* have finished with this object */
    fn dtor(&mut self, index: &X, item: T);

    /* Called whenever ready changed either from inside set or ping */
    fn ready_changed(&mut self, _items: &mut dyn Iterator<Item=(&X,&T)>) {}

    /* Called in ping to check if pending items are now ready */
    fn init(&mut self, index: &X, item: &mut P) -> Option<T>;

    /* Nothing is pending any more (edge-triggered event) */
    fn quiet(&mut self, _items: &mut dyn Iterator<Item=(&X,&T)>) {} 
}

#[derive(Clone,PartialEq,Eq)]
pub struct PartyState(u64,u64);

impl PartyState {
    pub(crate) fn null() -> PartyState { PartyState(0,0) }
    fn advance(&mut self) { self.1 += 1; }
}

pub struct Party<X: Eq+Hash+Clone,P,T,S: PartyActions<X,P,T>> {
    state: PartyState,
    want: HashSet<X>,
    pending: HashMap<X,P>,
    ready: HashMap<X,Option<T>>,
    actions: S
}

identitynumber!(IDS);

impl<X: Eq+Hash+Clone,P,T,S: PartyActions<X,P,T>> Party<X,P,T,S> {
    pub fn new(actions: S) -> Party<X,P,T,S> {
        Party {
            state: PartyState(IDS.next(),1),
            want: HashSet::new(),
            ready: HashMap::new(),
            pending: HashMap::new(),
            actions
        }
    }

    fn make_it_so(&mut self) {
        let mut ready_changed = false;
        let present = self.ready.keys().cloned().collect::<HashSet<_>>();
        /* calculate differences */
        let missings = self.want.difference(&present);
        let unneededs = present.difference(&self.want);
        /* Add what is missing */
        for missing in missings {
            let new = self.actions.ctor(missing);
            ready_changed = true;
            self.pending.insert(missing.clone(),new);
            self.ready.insert(missing.clone(),None);
        }
        /* Remove excess */
        for unneeded in unneededs {
            if let Some(old) = self.ready.remove(unneeded).unwrap() {
                self.actions.dtor(unneeded,old);
                ready_changed = true;
            }
        }
        /* Call steady if changed */
        if ready_changed {
            self.state.advance();
            let mut ready = self.ready.iter().filter_map(|(x,t)| t.as_ref().map(|t| (x,t)));
            self.actions.ready_changed(&mut ready);
        }
    }

    pub fn wanted(&self) -> &HashSet<X> { &self.want }

    pub fn set<V>(&mut self, wanted: V) where V: Iterator<Item=X> {
        let new = HashSet::from_iter(wanted);
        let old = mem::replace(&mut self.want,new);
        if old != self.want {
            self.make_it_so();
            self.ping();
        }
    }

    pub fn ping(&mut self) {
        let mut ready_changed = false;
        let mut gone = vec![];
        for (index,item) in self.pending.iter_mut() {
            if self.want.contains(index) {
                if let Some(value) = self.actions.init(index,item) {
                    self.ready.insert(index.clone(),Some(value));
                    ready_changed = true;
                    gone.push(index.clone());
                }
            } else {
                gone.push(index.clone());
            }
        }
        let was_pending = self.pending.len() > 0;
        for index in gone.drain(..) {
            self.pending.remove(&index);
        }
        if ready_changed {
            self.state.advance();
            let mut ready = self.ready.iter().filter_map(|(x,t)| t.as_ref().map(|t| (x,t)));
            self.actions.ready_changed(&mut ready);
        }
        if was_pending && self.pending.len() == 0 {
            self.actions.quiet(&mut self.ready.iter().map(|(x,y)| (x,y.as_ref().unwrap())))
        }
    }

    pub fn inner(&self) -> &S { &self.actions }
    pub fn inner_mut(&mut self) -> &mut S { &mut self.actions }
    pub fn is_ready(&self) -> bool { self.pending.len() == 0 }
    pub fn state(&self) -> PartyState { self.state.clone() }

    pub fn get(&self, index: X) -> Option<&T> {
        self.ready.get(&index).map(|x| x.as_ref()).flatten()
    }

    pub fn iter(&self) -> impl Iterator<Item=(&X,&T)> {
        self.ready.iter().filter_map(|(x,t)| t.as_ref().map(|t| (x,t)))
    }
}

impl<X: Eq+Hash+Clone,P,T,S: PartyActions<X,P,T>> Drop for Party<X,P,T,S> {
    fn drop(&mut self) {
        for (index,item) in self.ready.drain() {
            if let Some(item) = item {
                self.actions.dtor(&index,item);
            }
        }
        self.ready.clear();
    }
}
