use std::sync::{ Arc, Mutex };
use crate::domcss::dom::PeregrineDom;
use crate::input::InputEventKind;
use crate::input::low::modifiers::KeyboardModifiers;
use crate::input::translate::targetreporter::TargetReporter;
use crate::shape::spectres::spectremanager::{SpectreManager};
use crate::stage::stage::ReadStage;
use crate::webgl::global::WebGlGlobal;
use crate::{PgCommanderWeb, run::PgPeregrineConfig};
use crate::util::Message;
use super::modifiers::Modifiers;
use super::{event::EventSystem, keyboardinput::{KeyboardEventHandler, keyboard_events}, mouseinput::mouse_events};
use super::mapping::{ InputMapBuilder };
use super::mouseinput::{ MouseEventHandler };
use crate::input::{ InputEvent };
use super::mapping::InputMap;
use js_sys::Date;
use peregrine_data::{Commander };
use peregrine_toolkit::plumbing::distributor::Distributor;
use peregrine_toolkit_async::sync::needed::Needed;
use super::pointer::cursor::{ Cursor, CursorHandle };
use crate::run::CursorCircumstance;

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
    spectres: SpectreManager,
    pointer_last_seen: Arc<Mutex<Option<(f64,f64)>>>,
    target_reporter: TargetReporter
}

impl LowLevelState {
    fn new(dom: &PeregrineDom, commander: &PgCommanderWeb, spectres: &SpectreManager, config: &PgPeregrineConfig, target_reporter: &TargetReporter) -> Result<(LowLevelState,Distributor<InputEvent>),Message> {
        let mut mapping = InputMapBuilder::new();
        mapping.add_config(config)?;
        let modifiers = Arc::new(Mutex::new(Modifiers::new(KeyboardModifiers::new(false,false,false),&[])));
        let distributor = Distributor::new();
        Ok((LowLevelState {
            cursor: Cursor::new(dom,config)?,
            dom: dom.clone(),
            commander: commander.clone(),
            distributor: distributor.clone(),
            mapping: mapping.build(),
            modifiers,
            stage: Arc::new(Mutex::new(None)),
            spectres: spectres.clone(),
            pointer_last_seen: Arc::new(Mutex::new(None)),
            target_reporter: target_reporter.clone() 
        },distributor))
    }

    pub fn target_reporter(&self) -> &TargetReporter {
        &self.target_reporter
    }

    pub(super) fn set_pointer_last_seen(&mut self, position: (f64,f64)) {
        *self.pointer_last_seen.lock().unwrap() = Some(position);
    }

    pub(crate) fn pointer_last_seen(&self) -> Option<(f64,f64)> { 
        self.pointer_last_seen.lock().unwrap().clone()
    }

    fn update_stage(&self, stage: &ReadStage) { *self.stage.lock().unwrap() = Some(stage.clone()); }
    pub(super) fn stage(&self) -> Option<ReadStage> { self.stage.lock().unwrap().as_ref().cloned() }

    pub(super) fn update_keyboard_modifiers(&self, modifiers: KeyboardModifiers) {
        self.modifiers.lock().unwrap().update_keyboard_modifiers(modifiers);
    }

    pub(super) fn map(&self, key: &str, modifiers: &Modifiers) -> Vec<(InputEventKind,Vec<f64>)> {
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

    pub(crate) fn spectre_manager(&self) -> &SpectreManager { &self.spectres }
    pub(crate) fn spectre_manager_mut(&mut self) -> &mut SpectreManager { &mut self.spectres }

    pub fn set_artificial(&self, name: &str, start: bool) {
        self.modifiers.lock().unwrap().set_artificial(name,start);
    }
}

#[derive(Clone)]
pub struct LowLevelInput {
    #[allow(unused)]
    keyboard: EventSystem<KeyboardEventHandler>,
    #[allow(unused)]
    mouse: EventSystem<MouseEventHandler>,
    distributor: Distributor<InputEvent>,
    state: LowLevelState,
    mouse_moved: Needed,
    hotspot_cursor_handle: Option<Arc<CursorHandle>>
}

impl LowLevelInput {
    pub(crate) fn new(dom: &PeregrineDom, commander: &PgCommanderWeb, spectres: &SpectreManager, config: &PgPeregrineConfig, gl: &Arc<Mutex<WebGlGlobal>>, target_reporter: &TargetReporter) -> Result<LowLevelInput,Message> {
        let mouse_moved = Needed::new();
        let (state,distributor) = LowLevelState::new(dom,commander,spectres,config,target_reporter)?;
        let keyboard = keyboard_events(&state)?;
        let mouse = mouse_events(config,&state,gl,&mouse_moved)?;
        Ok(LowLevelInput { keyboard, mouse, distributor, state, mouse_moved, hotspot_cursor_handle: None })
    }

    pub fn distributor_mut(&mut self) -> &mut Distributor<InputEvent> { &mut self.distributor }

    pub fn update_stage(&self, stage: &ReadStage) { self.state.update_stage(stage); }

    pub fn set_artificial(&self, name: &str, start: bool) { self.state.set_artificial(name,start); }
    pub fn pointer_last_seen(&self) -> Option<(f64,f64)> { self.state.pointer_last_seen() }

    pub fn get_mouse_move_waiter(&self) -> Needed { self.mouse_moved.clone() }

    pub fn set_hotspot(&mut self, yn: bool) {
        if yn {
            self.hotspot_cursor_handle = Some(Arc::new(self.state.set_cursor(&CursorCircumstance::Hotspot)));
        } else {
            self.hotspot_cursor_handle = None;
        }
    }
}
