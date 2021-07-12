use std::sync::{ Arc, Mutex };
use crate::PeregrineInnerAPI;
use crate::shape::core::spectre::Spectre;
use crate::stage::stage::ReadStage;
use crate::{PeregrineAPI, PeregrineDom, run::PgPeregrineConfig, PgCommanderWeb };
use crate::util::Message;
use crate::input::low::lowlevel::LowLevelInput;
use crate::input::translate::Physics;
use crate::input::translate::debug::debug_register;

// XXX to  util
#[derive(Clone)]
pub struct Distributor<T>(Arc<Mutex<Vec<Box<dyn Fn(&T) + 'static>>>>);

impl<T> Distributor<T> {
    pub fn new() -> Distributor<T> {
        Distributor(Arc::new(Mutex::new(vec![])))
    }

    pub fn add<F>(&mut self, cb: F) where F: Fn(&T) + 'static {
        self.0.lock().unwrap().push(Box::new(cb));
    }

    pub fn send(&self, value: T) {
        let streams = self.0.lock().unwrap();
        for stream in streams.iter() {
            stream(&value);
        }
    }
}

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub enum InputEventKind {
    PullLeft,
    PullRight,
    PullIn,
    PullOut,
    PixelsLeft, // [pixels]
    PixelsRight, // [pixels]
    SetPosition, // [scale, centre, y]
    AnimatePosition, // [scale, centre, y]
    PixelsIn,
    PixelsOut,
    DebugAction,
}

impl InputEventKind {
    pub fn each() -> Vec<InputEventKind> {
        vec![
            InputEventKind::PullLeft,
            InputEventKind::PullRight,
            InputEventKind::PullIn,
            InputEventKind::PullOut,
            InputEventKind::PixelsLeft,
            InputEventKind::PixelsRight,
            InputEventKind::SetPosition,
            InputEventKind::AnimatePosition,
            InputEventKind::PixelsIn,
            InputEventKind::PixelsOut,
            InputEventKind::DebugAction
        ]
    }
}

#[derive(Debug,Clone)]
pub struct InputEvent {
    pub details: InputEventKind,
    pub start: bool,
    pub amount: Vec<f64>,
    pub timestamp_ms: f64
}

struct InputState {
    low_level: LowLevelInput,
    physics: Physics
}

#[derive(Clone)]
pub struct Input {
    low_level: Arc<Mutex<Option<InputState>>>
}

impl Input {
    pub fn new() ->Input {
        Input {
            low_level: Arc::new(Mutex::new(None))
        }
    }

    fn state<F,T>(&self, f: F) -> T where F: FnOnce(&InputState) -> T { f(self.low_level.lock().unwrap().as_ref().unwrap()) }

    pub fn set_api(&mut self, dom: &PeregrineDom, config: &PgPeregrineConfig, inner_api: &PeregrineInnerAPI, commander: &PgCommanderWeb) -> Result<(),Message> {
        let spectres = inner_api.spectres();
        let mut low_level = LowLevelInput::new(dom,commander,spectres,config)?;
        let physics = Physics::new(config,&mut low_level,inner_api,commander)?;
        debug_register(config,&mut low_level,inner_api)?;
        *self.low_level.lock().unwrap() = Some(InputState {
            low_level, physics
        });
        Ok(())
    }

    pub fn update_stage(&self, stage: &ReadStage) { self.state(|state| state.low_level.update_stage(stage)); }
    pub(crate) fn get_spectres(&self) -> Vec<Spectre> { self.state(|state| state.low_level.get_spectres()) }
    pub fn set_artificial(&self, name: &str, start: bool) { self.state(|state| state.low_level.set_artificial(name,start)); }

    pub(crate) fn goto(&self, api: &mut PeregrineInnerAPI, centre: f64, scale: f64) -> Result<(),Message> {
        self.state(|state| state.physics.goto(api,centre,scale))
    }
}
