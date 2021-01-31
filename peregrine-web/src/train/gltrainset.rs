use anyhow::{ bail };
use blackbox::blackbox_log;
use std::collections::HashMap;
use std::sync::{ Arc, Mutex };
use peregrine_core::{ Carriage, CarriageSpeed, PeregrineConfig, PeregrineApi };
use super::gltrain::GLTrain;
use crate::shape::layers::programstore::ProgramStore;
use web_sys::{ HtmlCanvasElement, WebGlRenderingContext };

#[derive(Clone)]
enum FadeState {
    Constant(Option<u32>),
    Fading(Option<u32>,u32,CarriageSpeed,f64)
}

struct GlTrainSetData {
    programs: ProgramStore,
    slow_fade_time: f64,
    fast_fade_time: f64,
    trains: HashMap<u32,GLTrain>,
    fade_state: FadeState
}

impl GlTrainSetData {
    fn new(config: &PeregrineConfig, context: &WebGlRenderingContext) -> GlTrainSetData {
        let programs = ProgramStore::new(context);
        GlTrainSetData {
            programs,
            slow_fade_time: config.get_f64("animate.fade.slow").unwrap_or(0.),
            fast_fade_time: config.get_f64("animate.fade.fast").unwrap_or(0.),
            trains: HashMap::new(),
            fade_state: FadeState::Constant(None)
        }
    }

    fn get_train(&mut self, index: u32) -> &mut GLTrain {
        if !self.trains.contains_key(&index) {
            self.trains.insert(index,GLTrain::new(index,&self.programs));
        }
        self.trains.get_mut(&index).unwrap()
    }

    fn set_carriages(&mut self, new_carriages: &[Carriage], index: u32) -> anyhow::Result<()> {
        self.get_train(index).set_carriages(new_carriages)
    }

    fn set_max(&mut self, index: u32, len: u64) {
        self.get_train(index).set_max(len);
    }

    fn start_fade(&mut self, index: u32, speed: CarriageSpeed) -> anyhow::Result<()> {
        let from = match self.fade_state {
            FadeState::Constant(x) => x,
            FadeState::Fading(_,_,_,_) => {
                bail!("overlapping fades sent to UI");
            }
        };
        self.fade_state = FadeState::Fading(from,index,speed,0.);
        Ok(())
    }

    fn fade_time(&self, speed: &CarriageSpeed, elapsed: f64) -> f64 {
        let fade_time = match speed {
            CarriageSpeed::Quick => self.fast_fade_time,
            CarriageSpeed::Slow => self.slow_fade_time
        };
        (elapsed/fade_time).min(1.).max(0.)
    }

    fn notify_faded_out(&mut self, index: u32) {
        self.get_train(index).discard();
    }

    fn notify_fade_state(&mut self) {
        match self.fade_state.clone() {
            FadeState::Constant(None) => {},
            FadeState::Constant(Some(index)) => {
                self.get_train(index).set_opacity(1.);
            },
            FadeState::Fading(from,to,speed,elapsed) => {
                let prop = self.fade_time(&speed,elapsed);
                self.get_train(to).set_opacity(prop);
                if let Some(from) = from {
                    self.get_train(from).set_opacity(1.-prop);
                }
            }
        }
    }

    fn animate_tick(&mut self, newly_elapsed: f64) -> bool {
        let mut complete = false;
        match self.fade_state.clone() {
            FadeState::Constant(_) => {}
            FadeState::Fading(from,to,speed,mut elapsed) => {
                elapsed += newly_elapsed;
                let prop = self.fade_time(&speed,elapsed);
                if prop >= 1. {
                    if let Some(from) = from {
                        self.notify_faded_out(from);
                    }
                    self.fade_state = FadeState::Constant(Some(to));
                    complete = true;
                } else {
                    self.fade_state = FadeState::Fading(from,to,speed.clone(),elapsed);
                }
                self.notify_fade_state();
            }
        }
        complete
    }
}

#[derive(Clone)]
pub struct GlTrainSet {
    data: Arc<Mutex<GlTrainSetData>>,
    api: PeregrineApi
}

impl GlTrainSet {
    pub fn new(config: &PeregrineConfig, api: PeregrineApi, context: &WebGlRenderingContext) -> GlTrainSet {
        GlTrainSet {
            api,
            data: Arc::new(Mutex::new(GlTrainSetData::new(config,context)))
        }
    }

    pub fn animate_tick(&mut self, newly_elapsed: f64) {
        if self.data.lock().unwrap().animate_tick(newly_elapsed) {
            blackbox_log!("gltrain","transition_complete()");
            self.api.transition_complete();
        }
    }

    pub fn set_carriages(&mut self, new_carriages: &[Carriage], index: u32) {
        self.data.lock().unwrap().set_carriages(new_carriages,index);
    }

    pub fn start_fade(&mut self, index: u32, max: u64, speed: CarriageSpeed) -> anyhow::Result<()> {
        self.data.lock().unwrap().start_fade(index,speed)?;
        self.data.lock().unwrap().set_max(index,max);
        Ok(())
    }
}