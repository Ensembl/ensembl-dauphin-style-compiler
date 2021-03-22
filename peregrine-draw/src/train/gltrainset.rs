use anyhow::{ bail };
use blackbox::blackbox_log;
use std::collections::HashMap;
use std::sync::{ Arc, Mutex };
use peregrine_data::{ Carriage, CarriageSpeed, PeregrineConfig, PeregrineCore };
use super::gltrain::GLTrain;
use crate::shape::core::stage::{ Stage, ReadStage };
use crate::shape::core::redrawneeded::{ RedrawNeeded, RedrawNeededLock };
use crate::webgl::DrawingSession;
use crate::webgl::global::WebGlGlobal;
use crate::shape::layers::drawingzmenus::ZMenuEvent;
use crate::util::message::Message;

#[derive(Clone)]
enum FadeState {
    Constant(Option<u32>),
    Fading(Option<u32>,u32,CarriageSpeed,f64,RedrawNeededLock)
}

struct GlTrainSetData {
    slow_fade_time: f64,
    fast_fade_time: f64,
    trains: HashMap<u32,GLTrain>,
    fade_state: FadeState,
    redraw_needed: RedrawNeeded
}

impl GlTrainSetData {
    fn new(config: &PeregrineConfig, redraw_needed: &RedrawNeeded) -> Result<GlTrainSetData,Message> {
        Ok(GlTrainSetData {
            slow_fade_time: config.get_f64("animate.fade.slow").unwrap_or(0.),
            fast_fade_time: config.get_f64("animate.fade.fast").unwrap_or(0.),
            trains: HashMap::new(),
            fade_state: FadeState::Constant(None),
            redraw_needed: redraw_needed.clone(),
        })
    }

    fn get_train(&mut self,gl: &WebGlGlobal, index: u32) -> &mut GLTrain {
        if !self.trains.contains_key(&index) {
            self.trains.insert(index,GLTrain::new(&gl.program_store(),&self.redraw_needed));
        }
        self.trains.get_mut(&index).unwrap()
    }

    fn set_carriages(&mut self, gl: &mut WebGlGlobal, new_carriages: &[Carriage], index: u32) -> Result<(),Message> {
        self.get_train(gl,index).set_carriages(new_carriages,gl)
    }

    fn set_max(&mut self, gl: &WebGlGlobal, index: u32, len: u64) {
        self.get_train(gl,index).set_max(len);
    }

    fn start_fade(&mut self, index: u32, speed: CarriageSpeed) -> Result<(),Message> {
        let from = match self.fade_state {
            FadeState::Constant(x) => x,
            FadeState::Fading(_,_,_,_,_) => {
                return Err(Message::XXXTmp(format!("overlapping fades sent to UI")));
            }
        };
        self.fade_state = FadeState::Fading(from,index,speed,0.,self.redraw_needed.clone().lock());
        Ok(())
    }

    fn fade_time(&self, speed: &CarriageSpeed, elapsed: f64) -> f64 {
        let fade_time = match speed {
            CarriageSpeed::Quick => self.fast_fade_time,
            CarriageSpeed::Slow => self.slow_fade_time
        };
        (elapsed/fade_time).min(1.).max(0.)
    }

    fn notify_fade_state(&mut self,gl: &WebGlGlobal) {
        match self.fade_state.clone() {
            FadeState::Constant(None) => {},
            FadeState::Constant(Some(index)) => {
                self.get_train(gl,index).set_opacity(1.);
            },
            FadeState::Fading(from,to,speed,elapsed,_) => {
                let prop = self.fade_time(&speed,elapsed);
                self.get_train(gl,to).set_opacity(prop);
                if let Some(from) = from {
                    self.get_train(gl,from).set_opacity(1.-prop);
                }
            }
        }
    }

    fn transition_animate_tick(&mut self, gl: &mut WebGlGlobal, newly_elapsed: f64) -> Result<bool,Message> {
        let mut complete = false;
        match self.fade_state.clone() {
            FadeState::Constant(_) => {}
            FadeState::Fading(from,to,speed,mut elapsed,redraw) => {
                elapsed += newly_elapsed;
                let prop = self.fade_time(&speed,elapsed);
                if prop >= 1. {
                    if let Some(from) = from {
                        self.get_train(gl,from).discard(gl).map_err(|e| Message::XXXTmp(e.to_string()))?;
                        self.trains.remove(&from);
                    }
                    self.fade_state = FadeState::Constant(Some(to));
                    self.redraw_needed.set(); // probably not needed; belt-and-braces
                    complete = true;
                } else {
                    self.fade_state = FadeState::Fading(from,to,speed.clone(),elapsed,redraw);
                }
                self.notify_fade_state(gl);
            }
        }
        Ok(complete)
    }

    fn draw_animate_tick(&mut self, stage: &ReadStage, gl: &mut WebGlGlobal) -> Result<(),Message> {
        let mut session = DrawingSession::new();
        session.begin(gl,stage).map_err(|e| Message::XXXTmp(e.to_string()))?;
        match self.fade_state.clone() {
            FadeState::Constant(None) => {},
            FadeState::Constant(Some(train)) => {
                self.get_train(gl,train).draw(gl,stage,&session).map_err(|e| Message::XXXTmp(e.to_string()))?;
            },
            FadeState::Fading(from,to,_,_,_) => {
                if let Some(from) = from {
                    self.get_train(gl,from).draw(gl,stage,&session).map_err(|e| Message::XXXTmp(e.to_string()))?;
                }
                self.get_train(gl,to).draw(gl,stage,&session).map_err(|e| Message::XXXTmp(e.to_string()))?;
            },
        }
        session.finish().map_err(|e| Message::XXXTmp(e.to_string()))?;
        Ok(())
    }

    fn intersects(&mut self, stage: &ReadStage,  gl: &mut WebGlGlobal, mouse: (u32,u32)) -> Result<Option<ZMenuEvent>,Message> {
        Ok(match self.fade_state {
            FadeState::Constant(x) => x,
            FadeState::Fading(_,x,_,_,_) => Some(x)
        }.map(|id| {
            self.get_train(gl,id).intersects(stage,mouse)
        }).transpose()?.flatten())
    }

    fn intersects_fast(&mut self, stage: &ReadStage, gl: &mut WebGlGlobal, mouse: (u32,u32)) -> Result<bool,Message> {
        Ok(match self.fade_state {
            FadeState::Constant(x) => x,
            FadeState::Fading(_,x,_,_,_) => Some(x)
        }.map(|id| {
            self.get_train(gl,id).intersects_fast(stage,mouse)
        }).transpose()?.unwrap_or(false))
    }

    fn discard(&mut self, gl: &mut WebGlGlobal) -> Result<(),Message> {
        for(_,mut train) in self.trains.drain() {
            train.discard(gl)?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct GlTrainSet {
    data: Arc<Mutex<GlTrainSetData>>
}

impl GlTrainSet {
    pub fn new(config: &PeregrineConfig, stage: &Stage) -> Result<GlTrainSet,Message> {
        Ok(GlTrainSet {
            data: Arc::new(Mutex::new(GlTrainSetData::new(config,&stage.redraw_needed())?))
        })
    }

    pub fn transition_animate_tick(&mut self, api: &PeregrineCore, gl: &mut WebGlGlobal, newly_elapsed: f64) -> Result<(),Message> {
        if self.data.lock().unwrap().transition_animate_tick(gl,newly_elapsed).map_err(|e| Message::XXXTmp(e.to_string()))? {
            blackbox_log!("gltrain","transition_complete()");
            api.transition_complete();
        }
        Ok(())
    }

    pub fn draw_animate_tick(&mut self, stage: &ReadStage, gl: &mut WebGlGlobal) -> Result<(),Message> {
        self.data.lock().unwrap().draw_animate_tick(stage,gl)
    }

    pub fn set_carriages(&mut self, new_carriages: &[Carriage], gl: &mut WebGlGlobal, index: u32) -> Result<(),Message> {
        self.data.lock().unwrap().set_carriages(gl,new_carriages,index)
    }

    pub fn start_fade(&mut self, gl: &WebGlGlobal, index: u32, max: u64, speed: CarriageSpeed) -> Result<(),Message> {
        self.data.lock().unwrap().start_fade(index,speed)?;
        self.data.lock().unwrap().set_max(gl,index,max);
        Ok(())
    }

    pub(crate) fn intersects(&self, stage: &ReadStage, gl: &mut WebGlGlobal, mouse: (u32,u32)) -> Result<Option<ZMenuEvent>,Message> {
        self.data.lock().unwrap().intersects(stage,gl,mouse)
    }

    pub(crate) fn intersects_fast(&self, stage: &ReadStage, gl: &mut WebGlGlobal, mouse: (u32,u32)) ->Result<bool,Message> {
        self.data.lock().unwrap().intersects_fast(stage,gl,mouse)
    }

    pub fn discard(&mut self, gl: &mut WebGlGlobal) -> Result<(),Message> {
        self.data.lock().unwrap().discard(gl)
    }
}