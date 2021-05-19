use std::sync::{ Arc, Mutex };
use crate::{PeregrineDom, run::PgPeregrineConfig};
use crate::util::Message;
use super::keyboardinput::{ KeyboardInput, KeyboardMap, KeyboardMapBuilder };
use crate::input::{ InputEvent, Distributor };

#[derive(Debug,Clone,Hash,PartialEq,Eq)]
pub struct Modifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool
}

#[derive(Debug,Clone,Hash,PartialEq,Eq)]
pub struct Key {
    pub text: String,
    pub modifiers: Modifiers
}

#[derive(Clone)]
pub struct LowLevelInput {
    keyboard: KeyboardInput,
    distributor: Distributor<InputEvent>
}

impl LowLevelInput {
    pub fn new(dom: &PeregrineDom, config: &PgPeregrineConfig) -> Result<LowLevelInput,Message> {
        let mut mapping = KeyboardMapBuilder::new();
        mapping.add_config(config)?;
        let distributor = Distributor::new();
        let keyboard = KeyboardInput::new(&distributor,dom,&mapping.build())?;
        Ok(LowLevelInput { keyboard, distributor })
    }

    pub fn distributor_mut(&mut self) -> &mut Distributor<InputEvent> { &mut self.distributor }
}
