use std::marker::PhantomData;

#[derive(Clone)]
pub struct Enumerable(pub usize, pub usize);

impl EnumerableKey for Enumerable {
    fn enumerable(&self) -> Enumerable { self.clone() }
}

pub trait EnumerableKey {
    fn enumerable(&self) -> Enumerable;
}

impl PartialEq for Enumerable {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&self.1)
    }
}

impl PartialOrd for Enumerable {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

pub fn enumerable_compose(a: &dyn EnumerableKey, b: &dyn EnumerableKey) -> Enumerable {
    let a = a.enumerable();
    let b = b.enumerable();
    Enumerable(a.0*b.1+b.0,a.1*b.1)
}

pub struct EnumerableMap<K,V> where {
    key: PhantomData<K>,
    data: Vec<Option<V>>
}

impl<K: EnumerableKey,V> EnumerableMap<K,V> {
    pub fn new() -> EnumerableMap<K,V> {
        EnumerableMap {
            key: PhantomData,
            data: vec![]
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        let e = key.enumerable();
        self.data.resize_with(e.0+1,Default::default);
        self.data[e.0] = Some(value);
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let e = key.enumerable();
        self.data.get(e.0).map(|x| x.as_ref()).flatten()
    }
}
