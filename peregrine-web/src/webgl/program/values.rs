use anyhow::{ anyhow as err, bail };
use std::collections::HashMap;
use super::attribute::Attribute;
use super::uniform::Uniform;
use std::marker::PhantomData;
use web_sys::{ WebGlUniformLocation, WebGlRenderingContext, WebGlBuffer };

pub(crate) trait ProcessValueType {
    type GLKey;
    type GLValue;
    type OurValue;

    fn name(&self) -> &str;
    fn activate(&self, context: &WebGlRenderingContext, gl_key: &Self::GLKey, gl_value: &Self::GLValue) -> anyhow::Result<()>;
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

struct ProcessValueEntry<GLKey,GLValue,OurValue> {
    gl_key: GLKey,
    gl_value: Option<GLValue>,
    object: Box<dyn ProcessValueType<GLKey=GLKey,GLValue=GLValue,OurValue=OurValue>>
}

pub(super) struct ProcessValues<GLKey,GLValue,OurKey : ProcessValueHandle,OurValue> {
    our_keys: HashMap<String,OurKey>,
    entries: Vec<ProcessValueEntry<GLKey,GLValue,OurValue>>
}

impl<GLKey,GLValue,OurKey : ProcessValueHandle,OurValue> ProcessValues<GLKey,GLValue,OurKey,OurValue> {
    pub(super) fn new() -> ProcessValues<GLKey,GLValue,OurKey,OurValue> {
        ProcessValues {
            our_keys: HashMap::new(),
            entries: vec![]
        }
    }

    pub(super) fn add_entry(&mut self, name: &str, gl_key: GLKey, object: Box<dyn ProcessValueType<GLKey=GLKey,GLValue=GLValue,OurValue=OurValue>>) -> OurKey {
        let idx = self.entries.len();
        self.our_keys.insert(name.to_string(),OurKey::new(idx));
        self.entries.push(ProcessValueEntry {
            gl_key, object,
            gl_value: None
        });
        OurKey::new(idx)
    }

    pub(super) fn activate_all(&self, context: &WebGlRenderingContext) -> anyhow::Result<()> {
        for (i,entry) in self.entries.iter().enumerate() {
            if let Some(buffer) = &entry.gl_value {
                entry.object.activate(context,&entry.gl_key,buffer)?;
            }
        }
        Ok(())
    }

    pub fn get_handle(&mut self, name: &str) -> anyhow::Result<OurKey> {
        Ok(self.our_keys.get(name).ok_or_else(|| err!("no such item '{}",name))?.cloned())
    }

    pub fn set_value(&mut self, context: &WebGlRenderingContext, our_key: &OurKey, our_value: OurValue) -> anyhow::Result<()> {
        let entry = &mut self.entries[our_key.get()];
        entry.gl_value = Some(entry.object.value_to_gl(context,our_value)?);
        Ok(())
    }

    pub fn delete(&mut self, context: &WebGlRenderingContext) -> anyhow::Result<()> {
        for entry in self.entries.iter() {
            if let Some(gl_value) = &entry.gl_value {
                entry.object.delete(context,gl_value)?;
            }
        }
        Ok(())
    }
}

process_value_handle!(AnonHandle);

pub(super) struct AnonProcessValues<GLKey,GLValue,OurValue>(ProcessValues<GLKey,GLValue,AnonHandle,OurValue>);

impl<GLKey,GLValue,OurValue> AnonProcessValues<GLKey,GLValue,OurValue> {
    pub(super) fn new() -> AnonProcessValues<GLKey,GLValue,OurValue> {
        AnonProcessValues(ProcessValues::new())
    }

    pub fn add_anon(&mut self, context: &WebGlRenderingContext, object: Box<dyn ProcessValueType<GLKey=GLKey,GLValue=GLValue,OurValue=OurValue>>,
                    gl_key: GLKey, our_value: OurValue) -> anyhow::Result<()> {
        let handle = self.0.add_entry("",gl_key,object);
        self.0.set_value(context,&handle,our_value)
    }

    pub(super) fn activate_all(&self, context: &WebGlRenderingContext) -> anyhow::Result<()> {
        self.0.activate_all(context)
    }

    pub fn delete(&mut self, context: &WebGlRenderingContext) -> anyhow::Result<()> {
        self.0.delete(context)
    }
}

