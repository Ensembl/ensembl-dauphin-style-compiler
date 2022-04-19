use std::{hash::Hash, collections::{HashMap, HashSet}, mem, iter::FromIterator};

pub trait SliderActions<X,P,T> {
    fn ctor(&mut self, index: &X) -> P;
    fn init(&mut self, index: &X, item: &mut P) -> Option<T>;
    fn dtor(&mut self, index: &X, item: T);
    fn done(&mut self, _items: &mut dyn Iterator<Item=(&X,&T)>) {} 
}

pub struct Slider<X: Eq+Hash+Clone,P,T,S: SliderActions<X,P,T>> {
    want: HashSet<X>,
    pending: HashMap<X,P>,
    ready: HashMap<X,Option<T>>,
    actions: S
}

impl<X: Eq+Hash+Clone,P,T,S: SliderActions<X,P,T>> Slider<X,P,T,S> {
    pub fn new(actions: S) -> Slider<X,P,T,S> {
        Slider {
            want: HashSet::new(),
            ready: HashMap::new(),
            pending: HashMap::new(),
            actions
        }
    }


    fn make_it_so(&mut self) {
        let present = self.ready.keys().cloned().collect::<HashSet<_>>();
        /* calculate differences */
        let missings = self.want.difference(&present);
        let unneededs = present.difference(&self.want);
        /* Add what is missing */
        for missing in missings {
            let new = self.actions.ctor(missing);
            self.pending.insert(missing.clone(),new);
            self.ready.insert(missing.clone(),None);
        }
        /* Remove excess */
        for unneeded in unneededs {
            if let Some(old) = self.ready.remove(unneeded).unwrap() {
                self.actions.dtor(unneeded,old);
            }
        }
    }

    pub fn wanted(&self) -> &HashSet<X> { &self.want }

    pub fn set<V>(&mut self, wanted: V) where V: Iterator<Item=X> {
        let new = HashSet::from_iter(wanted);
        let old = mem::replace(&mut self.want,new);
        if old != self.want {
            self.make_it_so();
            self.check();
        }
    }

    pub fn check(&mut self) {
        let mut gone = vec![];
        for (index,item) in self.pending.iter_mut() {
            if let Some(value) = self.actions.init(index,item) {
                self.ready.insert(index.clone(),Some(value));
                gone.push(index.clone());
            }
        }
        let was_pending = self.pending.len() > 0;
        for index in gone.drain(..) {
            self.pending.remove(&index);
        }
        if was_pending && self.pending.len() == 0 {
            self.actions.done(&mut self.ready.iter().map(|(x,y)| (x,y.as_ref().unwrap())))
        }
    }

    pub fn inner(&self) -> &S { &self.actions }
    pub fn inner_mut(&mut self) -> &mut S { &mut self.actions }
    pub fn is_ready(&self) -> bool { self.pending.len() == 0 }

    pub fn get(&self, index: X) -> Option<&T> {
        self.ready.get(&index).map(|x| x.as_ref()).flatten()
    }

    pub fn iter(&self) -> impl Iterator<Item=(&X,&T)> {
        self.ready.iter().filter_map(|(x,t)| t.as_ref().map(|t| (x,t)))
    }
}

impl<X: Eq+Hash+Clone,P,T,S: SliderActions<X,P,T>> Drop for Slider<X,P,T,S> {
    fn drop(&mut self) {
        for (index,item) in self.ready.drain() {
            if let Some(item) = item {
                self.actions.dtor(&index,item);
            }
        }
        self.ready.clear();
        self.actions.done(&mut self.ready.iter().map(|(x,y)| (x,y.as_ref().unwrap())));
    }
}
