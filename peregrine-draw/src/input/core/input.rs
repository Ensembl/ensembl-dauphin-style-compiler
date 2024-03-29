use std::sync::{ Arc, Mutex };
use peregrine_data::{PeregrineCore, SpecialClick, SingleHotspotEntry};
use peregrine_toolkit_async::sync::blocker::{Blocker, Lockout};

use crate::PeregrineInnerAPI;
use crate::domcss::dom::PeregrineDom;
use crate::input::translate::targetreporter::TargetReporter;
use crate::input::translate::translatehotspots::{translate_hotspots};
use crate::stage::stage::ReadStage;
use crate::webgl::global::WebGlGlobal;
use crate::{ run::PgPeregrineConfig, PgCommanderWeb };
use crate::util::Message;
use crate::input::low::lowlevel::LowLevelInput;
use crate::input::translate::InputTranslator;
use crate::input::translate::debug::debug_register;

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub enum InputEventKind {
    PullLeft,
    PullRight,
    PullIn,
    PullOut,
    PixelsLeft, // [pixels]
    PixelsRight, // [pixels]
    PixelsUp,
    PixelsDown,
    SetPosition, // [scale, centre, y]
    AnimatePosition, // [scale, centre, y]
    PixelsIn,
    PixelsOut,
    DebugAction,
    ZMenu,
    HoverChange
}

impl InputEventKind {
    pub fn each() -> Vec<InputEventKind> {
        vec![
            InputEventKind::PullLeft,
            InputEventKind::PullRight,
            InputEventKind::PullIn,
            InputEventKind::PullOut,
            InputEventKind::PixelsUp,
            InputEventKind::PixelsDown,
            InputEventKind::PixelsLeft,
            InputEventKind::PixelsRight,
            InputEventKind::SetPosition,
            InputEventKind::AnimatePosition,
            InputEventKind::PixelsIn,
            InputEventKind::PixelsOut,
            InputEventKind::DebugAction,
            InputEventKind::ZMenu,
            InputEventKind::HoverChange,
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
    translator: InputTranslator,
    inner_api: PeregrineInnerAPI,
    target_reporter: TargetReporter,
    stage: Option<ReadStage>
}

#[derive(Clone)]
pub struct Input {
    state: Arc<Mutex<Option<InputState>>>,
    queue_blocker: Blocker
}

impl Input {
    pub fn new(queue_blocker: &Blocker) -> Input {
        Input {
            state: Arc::new(Mutex::new(None)),
            queue_blocker: queue_blocker.clone()
        }
    }

    fn state<F,T>(&self, f: F) -> T where F: FnOnce(&mut InputState) -> T { f(self.state.lock().unwrap().as_mut().unwrap()) }

    pub(crate) fn set_api(&mut self, dom: &PeregrineDom, config: &PgPeregrineConfig, inner_api: &PeregrineInnerAPI, commander: &PgCommanderWeb, target_reporter: &TargetReporter, gl: &Arc<Mutex<WebGlGlobal>>) -> Result<(),Message> {
        let spectres = inner_api.spectres();
        let mut low_level = LowLevelInput::new(dom,commander,spectres,config,gl,&target_reporter)?;
        let translator = InputTranslator::new(config,&mut low_level,inner_api,commander,&self.queue_blocker,&target_reporter)?;
        translate_hotspots(&mut low_level,commander,&inner_api);
        debug_register(config,&mut low_level,inner_api)?;
        *self.state.lock().unwrap() = Some(InputState {
            low_level, translator,
            inner_api: inner_api.clone(),
            target_reporter: target_reporter.clone(),
            stage: None
        });
        Ok(())
    }

    pub fn set_limit(&self, limit: f64) {
        if let Some(state) = self.state.lock().unwrap().as_mut() {
            state.translator.set_limit(limit);
        }
    }

    pub fn set_hotspot(&self, yn: bool, hover: Vec<SingleHotspotEntry>, special: &[SpecialClick], position: (f64,f64)) {
        self.state(|state| state.low_level.set_hotspot(yn,hover,special,position));
    }

    pub fn update_stage(&self, stage: &ReadStage) { 
        self.state(|state| {
            state.stage = Some(stage.clone());
            state.low_level.update_stage(stage);
        });
    }

    pub(crate) fn get_pointer_last_seen(&self) -> Option<(f64,f64)> {
        self.state(|state| state.low_level.pointer_last_seen())
    }

    pub(crate) fn create_fake_mouse_move(&self) {
        let waiter = self.state(|state| state.low_level.get_mouse_move_waiter());
        waiter.set();
    }

    pub(crate) async fn wait_for_mouse_move(&self) {
        let waiter = self.state(|state| state.low_level.get_mouse_move_waiter());
        waiter.wait_until_needed().await;
    }

    pub fn set_artificial(&self, name: &str, start: bool) { self.state(|state| state.low_level.set_artificial(name,start)); }

    pub(crate) fn goto(&self, centre: f64, scale: f64, only_if_unknown: bool) -> Result<(),Message> {
        self.state(|state| state.translator.goto(&mut state.inner_api.clone(),centre,scale,only_if_unknown))
    }

    async fn jump_task(&self,data_api: PeregrineCore, location: String, lockout: Lockout) -> Result<(),Message> {
        if let Some((stick,centre,bp_per_screen)) = data_api.jump(&location).await {
            let slide = self.state(|state| { 
                let mut slide = false;
                if let Some(current_stick) = state.stage.as_ref().and_then(|s| s.stick()) {
                    if current_stick == &stick {
                        slide = true;
                    }
                }
                slide
            });
            if slide {
                self.goto(centre,bp_per_screen,false)?;
            } else {
                self.state(|state| {
                    state.inner_api.set_stick(&stick);
                    state.inner_api.set_position(Some(centre),Some(bp_per_screen),false);
                    state.target_reporter.force_report();
                });
            }
        }
        drop(lockout);
        Ok(())    
    }

    pub(crate) fn jump(&self, data_api: &PeregrineCore, commander: &PgCommanderWeb, location: &str) {
        let self2 = self.clone();
        let data_api = data_api.clone();
        let location = location.to_string();
        let lockout = self.queue_blocker.lock();
        commander.add("jump-web", 0, None, None, Box::pin(async move {
            self2.jump_task(data_api.clone(),location,lockout).await
        }));
    }
}
