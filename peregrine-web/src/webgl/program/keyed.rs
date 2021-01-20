use anyhow::{ anyhow as err, bail };
use std::collections::HashMap;
use super::attribute::Attribute;
use super::texture::Texture;
use super::uniform::Uniform;
use crate::webgl::canvas::canvas::Canvas;
use std::marker::PhantomData;
use web_sys::{ WebGlUniformLocation, WebGlRenderingContext, WebGlBuffer, WebGlTexture };

pub trait KetedHandle {
    fn new(value: usize) -> Self;
    fn get(&self) -> usize;
    fn cloned(&self) -> Self;
}

#[macro_export]
macro_rules! keyed_handle {
    ($name:ident) => {
        pub struct $name(usize);

        impl $crate::webgl::program::keyed::KetedHandle for $name {
            fn new(value: usize) -> Self { $name(value) }
            fn get(&self) -> usize { self.0 }
            fn cloned(&self) -> Self { $name(self.0) }
        }
    };
}

pub(super) struct KeyedValues<K: KetedHandle,T> {
    our_keys: HashMap<String,K>,
    entries: Vec<T>
}

impl<K: KetedHandle,T> KeyedValues<K,T> {
    pub fn new() -> KeyedValues<K,T> {
        KeyedValues {
            our_keys: HashMap::new(),
            entries: vec![]
        }
    }

    pub fn add(&mut self, key: &str, value: T) -> K {
        let idx = self.entries.len();
        self.our_keys.insert(key.to_string(),K::new(idx));
        self.entries.push(value);
        K::new(idx)
    }

    pub fn get_handle(&mut self, name: &str) -> anyhow::Result<K> {
        Ok(self.our_keys.get(name).ok_or_else(|| err!("no such item '{}",name))?.cloned())
    }

    pub fn get(&self, key: &K) -> &T { &self.entries[key.get()] }
    pub fn get_mut(&mut self, key: &K) -> &mut T { &mut self.entries[key.get()] }
    pub fn iter(&self) -> impl Iterator<Item=&T> { self.entries.iter() }
    pub fn iter_mut(&mut self) -> impl Iterator<Item=&mut T> { self.entries.iter_mut() }
}
