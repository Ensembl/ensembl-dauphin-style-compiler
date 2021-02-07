use anyhow::{ bail };
use blackbox::blackbox_log;
use std::collections::HashMap;
use std::sync::{ Arc, Mutex };
use peregrine_core::{ Carriage, CarriageSpeed, PeregrineConfig, PeregrineApi };
use super::gltrain::GLTrain;
use crate::shape::layers::programstore::ProgramStore;
use crate::shape::core::stage::{ Stage, RedrawNeeded };
use web_sys::{ WebGlRenderingContext };

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
    fade_state: FadeState,
    redraw_needed: RedrawNeeded
}

impl GlTrainSetData {
    fn new(config: &PeregrineConfig, redraw_needed: &RedrawNeeded, context: &WebGlRenderingContext) -> anyhow::Result<GlTrainSetData> {
        let programs = ProgramStore::new(context)?;
        Ok(GlTrainSetData {
            programs,
            slow_fade_time: config.get_f64("animate.fade.slow").unwrap_or(0.),
            fast_fade_time: config.get_f64("animate.fade.fast").unwrap_or(0.),
            trains: HashMap::new(),
            fade_state: FadeState::Constant(None),
            redraw_needed: redraw_needed.clone()
        })
    }

    fn get_train(&mut self, index: u32) -> &mut GLTrain {
        if !self.trains.contains_key(&index) {
            self.trains.insert(index,GLTrain::new(&self.programs,&self.redraw_needed));
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

    fn transition_animate_tick(&mut self, newly_elapsed: f64) -> bool {
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
                    self.redraw_needed.set(); // probably not needed; belt-and-braces
                    complete = true;
                } else {
                    self.fade_state = FadeState::Fading(from,to,speed.clone(),elapsed);
                }
                self.notify_fade_state();
            }
        }
        complete
    }

    fn draw_animate_tick(&mut self, stage: &Stage) -> anyhow::Result<()> {
        match self.fade_state.clone() {
            FadeState::Constant(None) => {},
            FadeState::Constant(Some(train)) => {
                self.get_train(train).draw(stage)?;
            },
            FadeState::Fading(from,to,_,_) => {
                if let Some(from) = from {
                    self.get_train(from).draw(stage)?;
                }
                self.get_train(to).draw(stage)?;
            },
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct GlTrainSet {
    data: Arc<Mutex<GlTrainSetData>>,
    api: PeregrineApi
}

impl GlTrainSet {
    pub fn new(config: &PeregrineConfig, api: PeregrineApi, stage: &Stage, context: &WebGlRenderingContext) -> anyhow::Result<GlTrainSet> {
        Ok(GlTrainSet {
            api,
            data: Arc::new(Mutex::new(GlTrainSetData::new(config,&stage.redraw_needed(),context)?))
        })
    }

    pub fn transition_animate_tick(&mut self, newly_elapsed: f64) {
        if self.data.lock().unwrap().transition_animate_tick(newly_elapsed) {
            blackbox_log!("gltrain","transition_complete()");
            self.api.transition_complete();
        }
    }

    pub fn draw_animate_tick(&mut self, stage: &Stage) -> anyhow::Result<()> {
        self.data.lock().unwrap().draw_animate_tick(stage)
    }

    pub fn set_carriages(&mut self, new_carriages: &[Carriage], index: u32) -> anyhow::Result<()> {
        self.data.lock().unwrap().set_carriages(new_carriages,index)
    }

    pub fn start_fade(&mut self, index: u32, max: u64, speed: CarriageSpeed) -> anyhow::Result<()> {
        self.data.lock().unwrap().start_fade(index,speed)?;
        self.data.lock().unwrap().set_max(index,max);
        Ok(())
    }
}