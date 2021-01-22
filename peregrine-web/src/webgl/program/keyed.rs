use anyhow::{ anyhow as err, bail };
use std::collections::HashMap;
use std::marker::PhantomData;
use super::attribute::Attribute;
use super::texture::Texture;
use super::uniform::Uniform;
use crate::webgl::canvas::canvas::Canvas;
use web_sys::{ WebGlUniformLocation, WebGlRenderingContext, WebGlBuffer, WebGlTexture };

pub trait KeyedHandle {
    fn new(value: usize) -> Self;
    fn get(&self) -> usize;
    fn cloned(&self) -> Self;
}

#[macro_export]
macro_rules! keyed_handle {
    ($name:ident) => {
        pub struct $name(usize);

        impl $crate::webgl::program::keyed::KeyedHandle for $name {
            fn new(value: usize) -> Self { $name(value) }
            fn get(&self) -> usize { self.0 }
            fn cloned(&self) -> Self { $name(self.0) }
        }
    };
}

pub(crate) struct KeyedKeys<K: KeyedHandle,T>(HashMap<String,K>,PhantomData<T>);

impl<K: KeyedHandle, T> Clone for KeyedKeys<K,T> {
    fn clone(&self) -> Self {
        KeyedKeys(self.0.iter().map(|x| (x.0.to_string(),x.1.cloned())).collect(),PhantomData)
    }
}

impl<K: KeyedHandle, T> KeyedKeys<K,T> {
    fn new() -> KeyedKeys<K,T> {
        KeyedKeys(HashMap::new(),PhantomData)
    }

    fn insert(&mut self, name: &str, key: K) {
        self.0.insert(name.to_string(),key);
    }

    pub fn get_handle(&mut self, name: &str) -> anyhow::Result<K> {
        Ok(self.0.get(name).ok_or_else(|| err!("no such item '{}",name))?.cloned())
    }

    pub fn make_maker<'f,F,U>(&self, template: F) -> KeyedDataMaker<'f,K,U> where F: Fn() -> U + 'f {
        KeyedDataMaker(self.0.len(),Box::new(template),PhantomData)
    }
}

pub(crate) struct KeyedDataMaker<'f,K: KeyedHandle,T>(usize,Box<dyn Fn() -> T + 'f>,PhantomData<K>);

impl<'f,K: KeyedHandle,T> KeyedDataMaker<'f,K,T> {
    pub(crate) fn make(&self) -> KeyedData<K,T> {
        KeyedData((0..self.0).map(|_| self.1()).collect(),PhantomData)
    }
}

pub(crate) struct KeyedData<K: KeyedHandle, T>(Vec<T>,PhantomData<K>);

impl<K: KeyedHandle,T> KeyedData<K,T> {
    pub fn new() -> KeyedData<K,T> {
        KeyedData(vec![],PhantomData)
    }

    pub(crate) fn add(&mut self, value: T) -> K {
        let idx = self.0.len();
        self.0.push(value);
        K::new(idx)
    }

    pub fn get(&self, key: &K) -> &T { &self.0[key.get()] }
    pub fn get_mut(&mut self, key: &K) -> &mut T { &mut self.0[key.get()] }
    pub fn values(&self) -> impl Iterator<Item=&T> { self.0.iter() }
    pub fn values_mut(&mut self) -> impl Iterator<Item=&mut T> { self.0.iter_mut() }

    pub fn into<F,U,E>(mut self, f: F) -> Result<KeyedData<K,U>,E> where F: Fn(K,T) -> Result<U,E> {
        Ok(KeyedData(self.0.drain(..).enumerate().map(|(i,t)| f(K::new(i),t)).collect::<Result<_,_>>()?,PhantomData))
    }
}

pub(crate) struct KeyedValues<K: KeyedHandle,T> {
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

    pub(crate) fn keys(&self) -> KeyedKeys<K,T> {
        self.our_keys.clone()
    }

    pub fn add(&mut self, key: &str, value: T) -> K {
        let handle = self.entries.add(value);
        self.our_keys.insert(key,K::new(handle.get()));
        handle
    }

    pub fn get_handle(&mut self, name: &str) -> anyhow::Result<K> {
        self.our_keys.get_handle(name)
    }

    pub fn data(&self) -> &KeyedData<K,T> { &self.entries }
    pub fn data_mut(&mut self) -> &mut KeyedData<K,T> { &mut self.entries }
    pub fn into_data(self) -> KeyedData<K,T> { self.entries }
}
