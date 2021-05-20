use std::sync::{ Arc, Mutex };
use crate::{PeregrineDom, run::PgPeregrineConfig};
use crate::util::Message;
use super::keyboardinput::{ KeyboardInput, KeyboardMap, KeyboardMapBuilder };
use super::mouseinput::{ MouseInput, MouseMapBuilder };
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
    mouse: MouseInput,
    distributor: Distributor<InputEvent>
}

impl LowLevelInput {
    pub fn new(dom: &PeregrineDom, config: &PgPeregrineConfig) -> Result<LowLevelInput,Message> {
        let mut key_mapping = KeyboardMapBuilder::new();
        key_mapping.add_config(config)?;
        let mut mouse_mapping = MouseMapBuilder::new();
        //mouse_mapping.add_config(config)?;
        let distributor = Distributor::new();
        let keyboard = KeyboardInput::new(&distributor,dom,&key_mapping.build())?;
        let mouse = MouseInput::new(&distributor,dom,&mouse_mapping.build())?;
        Ok(LowLevelInput { keyboard, mouse, distributor })
    }

    pub fn distributor_mut(&mut self) -> &mut Distributor<InputEvent> { &mut self.distributor }
}
