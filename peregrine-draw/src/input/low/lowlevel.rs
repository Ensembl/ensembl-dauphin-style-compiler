use std::sync::{ Arc, Mutex };
use crate::input::InputEventKind;
use crate::shape::core::spectre::Spectre;
use crate::shape::core::spectremanager::{SpectreHandle, SpectreManager};
use crate::stage::stage::ReadStage;
use crate::{PeregrineDom, PgCommanderWeb, run::PgPeregrineConfig};
use crate::util::Message;
use super::{event::EventSystem, keyboardinput::{KeyboardEventHandler, keyboard_events}, mouseinput::mouse_events};
use super::mapping::{ InputMapBuilder };
use super::mouseinput::{ MouseEventHandler };
use crate::input::{ InputEvent, Distributor };
use super::mapping::InputMap;
use js_sys::Date;
use peregrine_data::{Commander, VariableValues};
use super::pointer::cursor::{ Cursor, CursorHandle };
use crate::run::CursorCircumstance;

#[derive(Debug,Clone,Hash,PartialEq,Eq)]
pub struct Modifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool
}

// XXX pub
#[derive(Clone)]
pub struct LowLevelState {
    commander: PgCommanderWeb,
    distributor: Distributor<InputEvent>,
    dom: PeregrineDom,
    mapping: InputMap,
    modifiers: Arc<Mutex<Modifiers>>,
    stage: Arc<Mutex<Option<ReadStage>>>,
    cursor: Cursor,
    spectres: SpectreManager
}

impl LowLevelState {
    fn new(dom: &PeregrineDom, commander: &PgCommanderWeb, spectres: &SpectreManager, config: &PgPeregrineConfig) -> Result<(LowLevelState,Distributor<InputEvent>),Message> {
        let mut mapping = InputMapBuilder::new();
        mapping.add_config(config)?;
        let modifiers = Arc::new(Mutex::new(Modifiers {
            shift: false,
            control: false,
            alt: false
        }));
        let distributor = Distributor::new();
        Ok((LowLevelState {
            cursor: Cursor::new(dom,config)?,
            dom: dom.clone(),
            commander: commander.clone(),
            distributor: distributor.clone(),
            mapping: mapping.build(),
            modifiers,
            stage: Arc::new(Mutex::new(None)),
            spectres: spectres.clone()
        },distributor))
    }

    fn update_stage(&self, stage: &ReadStage) { *self.stage.lock().unwrap() = Some(stage.clone()); }
    pub(super) fn stage(&self) -> Option<ReadStage> { self.stage.lock().unwrap().as_ref().cloned() }

    pub(super) fn update_modifiers(&self, modifiers: Modifiers) {
        *self.modifiers.lock().unwrap() = modifiers;
    }

    pub(super) fn map(&self, key: &str, modifiers: &Modifiers) -> Option<(InputEventKind,Vec<f64>)> {
        self.mapping.map(key,modifiers)
    }

    pub(super) fn dom(&self) -> &PeregrineDom { &self.dom }
    pub(super) fn modifiers(&self) -> Modifiers { self.modifiers.lock().unwrap().clone() }

    pub fn send(&self, kind: InputEventKind, start: bool, args: &[f64]) {
        self.distributor.send(InputEvent {
            details: kind,
            start,
            amount: args.to_vec(),
            timestamp_ms: Date::now()
        })
    }

    pub(super) fn commander(&self) -> &PgCommanderWeb { &self.commander }

    pub(super) fn timer<F>(&self, timeout: f64, cb: F) where F: FnOnce() + 'static {
        self.commander.executor().add_timer(timeout,Box::new(cb));
    }

    pub fn set_cursor(&self, circ: &CursorCircumstance) -> CursorHandle {
        self.cursor.set(circ)
    }

    pub(crate) fn add_spectre(&self, spectre: Spectre) -> SpectreHandle {
        self.spectres.add(spectre)
    }

    pub(crate) fn spectre_manager(&self) -> &SpectreManager { &self.spectres }
    /*
    pub(crate) fn get_spectres(&self) -> Vec<Spectre> { self.spectres.get_spectres() }
    pub(crate) fn redraw_spectres(&self) -> Result<(),Message> { self.spectres.update() }
    pub(crate) fn spectre_variables(&self) -> &VariableValues<f64> { self.spectres.variables() }
    */
}

#[derive(Clone)]
pub struct LowLevelInput {
    keyboard: EventSystem<KeyboardEventHandler>,
    mouse: EventSystem<MouseEventHandler>,
    distributor: Distributor<InputEvent>,
    state: LowLevelState
}

impl LowLevelInput {
    pub(crate) fn new(dom: &PeregrineDom, commander: &PgCommanderWeb, spectres: &SpectreManager, config: &PgPeregrineConfig) -> Result<LowLevelInput,Message> {
        let (state,distributor) = LowLevelState::new(dom,commander,spectres,config)?;
        let keyboard = keyboard_events(&state)?;
        let mouse = mouse_events(config,&state)?;
        Ok(LowLevelInput { keyboard, mouse, distributor, state })
    }

    pub fn distributor_mut(&mut self) -> &mut Distributor<InputEvent> { &mut self.distributor }

    pub fn update_stage(&self, stage: &ReadStage) { self.state.update_stage(stage); }
    pub(crate) fn get_spectres(&self) -> Vec<Spectre> { self.state.spectre_manager().get_spectres() }
}
