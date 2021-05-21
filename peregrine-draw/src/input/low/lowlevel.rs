use std::sync::{ Arc, Mutex };
use crate::{PeregrineDom, run::PgPeregrineConfig};
use crate::util::Message;
use super::keyboardinput::{ KeyboardInput };
use super::mapping::{ InputMapBuilder };
use super::mouseinput::{ MouseInput };
use crate::input::{ InputEvent, Distributor };
#[derive(Debug,Clone,Hash,PartialEq,Eq)]
pub struct Modifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool
}

#[derive(Clone)]
pub struct LowLevelInput {
    keyboard: KeyboardInput,
    mouse: MouseInput,
    distributor: Distributor<InputEvent>
}

impl LowLevelInput {
    pub fn new(dom: &PeregrineDom, config: &PgPeregrineConfig) -> Result<LowLevelInput,Message> {
        let mut mapping = InputMapBuilder::new();
        mapping.add_config(config)?;
        let modifiers = Arc::new(Mutex::new(Modifiers {
            shift: false,
            control: false,
            alt: false
        }));
        let distributor = Distributor::new();
        let mapping = mapping.build();
        let keyboard = KeyboardInput::new(&distributor,dom,&mapping,&modifiers)?;
        let mouse = MouseInput::new(&distributor,dom,&mapping,&modifiers)?;
        Ok(LowLevelInput { keyboard, mouse, distributor })
    }

    pub fn distributor_mut(&mut self) -> &mut Distributor<InputEvent> { &mut self.distributor }
}
