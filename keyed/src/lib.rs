use anyhow::{ anyhow as err };
use std::collections::{ BTreeSet, HashMap };
use std::marker::PhantomData;

pub trait KeyedHandle {
    fn new(value: usize) -> Self;
    fn get(&self) -> usize;
    fn clone_handle(&self) -> Self;
}

#[macro_export]
macro_rules! keyed_handle {
    ($name:ident) => {

        #[derive(PartialEq,Eq,Hash,Debug)] // XXX debug
        pub struct $name(usize);

        impl $crate::KeyedHandle for $name {
            fn new(value: usize) -> Self { $name(value) }
            fn get(&self) -> usize { self.0 }
            fn clone_handle(&self) -> Self { $name(self.0) }
        }

        impl Clone for $name where $name: $crate::KeyedHandle {
            fn clone(&self) -> Self {
                use $crate::KeyedHandle;
                self.clone_handle()
            }
        }
    };
}

pub struct KeyedKeys<K: KeyedHandle,T>(HashMap<String,K>,PhantomData<T>);

impl<K: KeyedHandle, T> Clone for KeyedKeys<K,T> {
    fn clone(&self) -> Self {
        KeyedKeys(self.0.iter().map(|x| (x.0.to_string(),x.1.clone_handle())).collect(),PhantomData)
    }
}

impl<K: KeyedHandle, T> KeyedKeys<K,T> {
    fn new() -> KeyedKeys<K,T> {
        KeyedKeys(HashMap::new(),PhantomData)
    }

    fn insert(&mut self, name: &str, key: K) {
        self.0.insert(name.to_string(),key);
    }

    pub fn iter(&self) -> impl Iterator<Item=(&String,&K)> {
        self.0.iter()
    }

    pub fn try_get_handle(&self, name: &str) -> Option<K> {
        self.0.get(name).map(|h| h.clone_handle())
    }

    pub fn get_handle(&self, name: &str) -> anyhow::Result<K> {
        Ok(self.try_get_handle(name).ok_or_else(|| err!("no such item '{}",name))?.clone_handle())
    }

    pub fn make_maker<'f,F,U>(&self, template: F) -> KeyedDataMaker<'f,K,U> where F: Fn() -> U + 'f {
        KeyedDataMaker(self.0.len(),Box::new(template),PhantomData)
    }

    fn map<U>(&self) -> KeyedKeys<K,U> {
        let mut out = HashMap::new();
        for (k,v) in self.0.iter() {
            out.insert(k.clone(),v.clone_handle());
        }
        KeyedKeys(out,PhantomData)
    }
}

pub struct KeyedDataMaker<'f,K: KeyedHandle,T>(usize,Box<dyn Fn() -> T + 'f>,PhantomData<K>);

impl<'f,K: KeyedHandle,T> KeyedDataMaker<'f,K,T> {
    pub fn make(&self) -> KeyedData<K,T> {
        KeyedData((0..self.0).map(|_| self.1()).collect(),PhantomData)
    }
}

pub struct KeyedData<K: KeyedHandle, T>(Vec<T>,PhantomData<K>);

impl<K: KeyedHandle,T> KeyedData<K,T> {
    pub fn new() -> KeyedData<K,T> {
        KeyedData(vec![],PhantomData)
    }

    pub fn add(&mut self, value: T) -> K {
        let idx = self.0.len();
        self.0.push(value);
        K::new(idx)
    }

    pub fn len(&self) -> usize { self.0.len()}
    pub fn get(&self, key: &K) -> &T { &self.0[key.get()] }
    pub fn get_mut(&mut self, key: &K) -> &mut T { &mut self.0[key.get()] }
    pub fn values(&self) -> impl Iterator<Item=&T> { self.0.iter() }
    pub fn values_mut(&mut self) -> impl Iterator<Item=&mut T> { self.0.iter_mut() }

    pub fn items(&self) -> impl Iterator<Item=(K,&T)> { self.values().enumerate().map(|(i,v)| (K::new(i),v)) }

    pub fn take(&mut self) -> Vec<(K,T)> { self.0.drain(..).enumerate().map(|(i,v)| (K::new(i),v)).collect() }

    pub fn map_into<F,U,E>(mut self, f: F) -> Result<KeyedData<K,U>,E> where F: Fn(K,T) -> Result<U,E> {
        Ok(KeyedData(self.0.drain(..).enumerate().map(|(i,t)| f(K::new(i),t)).collect::<Result<_,_>>()?,PhantomData))
    }

    pub fn map<F,U,E>(&self, mut f: F) -> Result<KeyedData<K,U>,E> where F: FnMut(K,&T) -> Result<U,E> {
        Ok(KeyedData(self.0.iter().enumerate().map(|(i,t)| f(K::new(i),t)).collect::<Result<_,_>>()?,PhantomData))
    }
}

impl<K: KeyedHandle,T> KeyedData<K,Option<T>> {
    pub fn insert(&mut self, index: &K, value: T) {
        while index.get() >= self.0.len() {
            self.0.push(None);
        }
        self.0[index.get()] = Some(value);
    }

    pub fn try_get(&self, index: &K) -> &Option<T> {
        if index.get() >= self.0.len() {
            return &None;
        }
        self.get(index)
    }

    pub fn remove(&mut self, index: &K) -> Option<T> {
        self.0[index.get()].take()
    }
}

pub struct KeyedValues<K: KeyedHandle,T> {
    our_keys: KeyedKeys<K,T>,
    entries: KeyedData<K,T>
}

impl<K: KeyedHandle,T> KeyedValues<K,T> {
    pub fn new() -> KeyedValues<K,T> {
        KeyedValues {
            our_keys: KeyedKeys::new(),
            entries: KeyedData::new()
        }
    }

    pub fn keys(&self) -> KeyedKeys<K,T> {
        self.our_keys.clone()
    }

    pub fn add(&mut self, key: &str, value: T) -> K {
        let handle = self.entries.add(value);
        self.our_keys.insert(key,K::new(handle.get()));
        handle
    }

    pub fn get_handle(&self, name: &str) -> anyhow::Result<K> { self.our_keys.get_handle(name) }
    pub fn try_get_handle(&self, name: &str) -> Option<K> { self.our_keys.try_get_handle(name) }

    pub fn map<F,U,E>(&self, f: F) -> Result<KeyedValues<K,U>,E> where F: Fn(K,&T) -> Result<U,E> {
        Ok(KeyedValues {
            our_keys: self.our_keys.map(),
            entries: self.entries.map(f)?
        })
    }

    pub fn data(&self) -> &KeyedData<K,T> { &self.entries }
    pub fn data_mut(&mut self) -> &mut KeyedData<K,T> { &mut self.entries }
}

// TODO ducument
// TODO test
