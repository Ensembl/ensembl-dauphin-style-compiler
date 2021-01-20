use anyhow::{ anyhow as err, bail };
use std::collections::HashMap;
use super::attribute::Attribute;
use super::uniform::Uniform;
use std::marker::PhantomData;
use web_sys::{ WebGlUniformLocation, WebGlRenderingContext, WebGlBuffer };

pub(crate) trait ProcessValueType {
    type GLValue;
    type OurValue;

    fn name(&self) -> &str;
    fn activate(&self, context: &WebGlRenderingContext, gl_value: &Self::GLValue) -> anyhow::Result<()>;
    fn value_to_gl(&self, context: &WebGlRenderingContext, our_data: Self::OurValue) -> anyhow::Result<Self::GLValue>;
    fn delete(&self, context: &WebGlRenderingContext, gl_value: &Self::GLValue) -> anyhow::Result<()>;
}

pub trait ProcessValueHandle {
    fn new(value: usize) -> Self;
    fn get(&self) -> usize;
    fn cloned(&self) -> Self;
}

#[macro_export]
macro_rules! process_value_handle {
    ($name:ident) => {
        pub struct $name(usize);

        impl $crate::webgl::program::values::ProcessValueHandle for $name {
            fn new(value: usize) -> Self { $name(value) }
            fn get(&self) -> usize { self.0 }
            fn cloned(&self) -> Self { $name(self.0) }
        }
    };
}

struct ProcessValueEntry<GLValue,OurValue> {
    gl_value: Option<GLValue>,
    object: Box<dyn ProcessValueType<GLValue=GLValue,OurValue=OurValue>>
}

pub(super) struct KeyedValues<K: ProcessValueHandle,T> {
    our_keys: HashMap<String,K>,
    entries: Vec<T>
}

impl<K: ProcessValueHandle,T> KeyedValues<K,T> {
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

pub(super) struct ProcessValues<GLValue,OurKey : ProcessValueHandle,OurValue>(KeyedValues<OurKey,ProcessValueEntry<GLValue,OurValue>>);

impl<GLValue,OurKey : ProcessValueHandle,OurValue> ProcessValues<GLValue,OurKey,OurValue> {
    pub(super) fn new() -> ProcessValues<GLValue,OurKey,OurValue> {
        ProcessValues(KeyedValues::new())
    }

    pub(super) fn add_entry(&mut self, name: &str, object: Box<dyn ProcessValueType<GLValue=GLValue,OurValue=OurValue>>) -> OurKey {
        self.0.add(name,ProcessValueEntry { object, gl_value: None })
    }

    pub(super) fn activate_all(&self, context: &WebGlRenderingContext) -> anyhow::Result<()> {
        for (i,entry) in self.0.iter().enumerate() {
            if let Some(buffer) = &entry.gl_value {
                entry.object.activate(context,buffer)?;
            }
        }
        Ok(())
    }

    pub fn get_handle(&mut self, name: &str) -> anyhow::Result<OurKey> {
        self.0.get_handle(name)
    }

    pub fn set_value(&mut self, context: &WebGlRenderingContext, our_key: &OurKey, our_value: OurValue) -> anyhow::Result<()> {
        let entry = self.0.get_mut(our_key);
        if let Some(old_value) = entry.gl_value.take() {
            entry.object.delete(context,&old_value)?;
        }
        entry.gl_value = Some(entry.object.value_to_gl(context,our_value)?);
        Ok(())
    }

    pub fn delete(&mut self, context: &WebGlRenderingContext) -> anyhow::Result<()> {
        for entry in self.0.iter() {
            if let Some(gl_value) = &entry.gl_value {
                entry.object.delete(context,gl_value)?;
            }
        }
        Ok(())
    }
}

process_value_handle!(AnonHandle);

pub(super) struct AnonProcessValues<GLValue,OurValue>(ProcessValues<GLValue,AnonHandle,OurValue>);

impl<GLValue,OurValue> AnonProcessValues<GLValue,OurValue> {
    pub(super) fn new() -> AnonProcessValues<GLValue,OurValue> {
        AnonProcessValues(ProcessValues::new())
    }

    pub fn add_anon(&mut self, context: &WebGlRenderingContext, object: Box<dyn ProcessValueType<GLValue=GLValue,OurValue=OurValue>>,
                    our_value: OurValue) -> anyhow::Result<()> {
        let handle = self.0.add_entry("",object);
        self.0.set_value(context,&handle,our_value)
    }

    pub(super) fn activate_all(&self, context: &WebGlRenderingContext) -> anyhow::Result<()> {
        self.0.activate_all(context)
    }

    pub fn delete(&mut self, context: &WebGlRenderingContext) -> anyhow::Result<()> {
        self.0.delete(context)
    }
}
