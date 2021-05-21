use std::sync::{ Arc, Mutex };
use web_sys::MouseEvent;

use crate::{PeregrineDom, run::PgPeregrineConfig};
use crate::util::Message;
use super::{event::EventSystem, keyboardinput::{KeyboardEventHandler, keyboard_events}, mouseinput::mouse_events};
use super::mapping::{ InputMapBuilder };
use super::mouseinput::{ MouseEventHandler };
use crate::input::{ InputEvent, Distributor };
#[derive(Debug,Clone,Hash,PartialEq,Eq)]
pub struct Modifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool
}

#[derive(Clone)]
pub struct LowLevelInput {
    keyboard: EventSystem<KeyboardEventHandler>,
    mouse: EventSystem<MouseEventHandler>,
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
        let keyboard = keyboard_events(&distributor,dom,&mapping,&modifiers)?;
        let mouse = mouse_events(&distributor,dom,&mapping,&modifiers)?;
        Ok(LowLevelInput { keyboard, mouse, distributor })
    }

    pub fn distributor_mut(&mut self) -> &mut Distributor<InputEvent> { &mut self.distributor }
}
