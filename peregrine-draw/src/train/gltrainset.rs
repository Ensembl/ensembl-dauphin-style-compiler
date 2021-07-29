use std::collections::HashMap;
use std::sync::{ Arc, Mutex };
use peregrine_data::{ Carriage, CarriageSpeed, PeregrineCore };
use peregrine_toolkit::sync::needed::{Needed, NeededLock};
use super::gltrain::GLTrain;
use crate::{run::{ PgPeregrineConfig, PgConfigKey }, stage::stage::{ Stage, ReadStage } };
use crate::webgl::DrawingSession;
use crate::webgl::global::WebGlGlobal;
use crate::shape::layers::drawingzmenus::ZMenuEvent;
use crate::util::message::Message;

#[derive(Clone)]
enum FadeState {
    Constant(Option<u32>),
    Fading(Option<u32>,u32,CarriageSpeed,Option<f64>,Arc<NeededLock>)
}

struct GlTrainSetData {
    slow_fade_time: f64,
    slow_cross_fade_time: f64,
    fast_fade_time: f64,
    slow_fade_overlap_prop: f64,
    slow_cross_fade_overlap_prop: f64,
    fast_fade_overlap_prop: f64,
    trains: HashMap<u32,GLTrain>,
    fade_state: FadeState,
    redraw_needed: Needed
}

impl GlTrainSetData {
    fn new(draw_config: &PgPeregrineConfig,redraw_needed: &Needed) -> Result<GlTrainSetData,Message> {
        Ok(GlTrainSetData {
            slow_fade_time: draw_config.get_f64(&PgConfigKey::AnimationFadeRate(CarriageSpeed::Slow))?,
            slow_cross_fade_time: draw_config.get_f64(&PgConfigKey::AnimationFadeRate(CarriageSpeed::SlowCrossFade))?,
            fast_fade_time: draw_config.get_f64(&PgConfigKey::AnimationFadeRate(CarriageSpeed::Quick))?,
            slow_fade_overlap_prop: draw_config.get_f64(&PgConfigKey::FadeOverlap(CarriageSpeed::Slow))?,
            slow_cross_fade_overlap_prop: draw_config.get_f64(&PgConfigKey::FadeOverlap(CarriageSpeed::SlowCrossFade))?,
            fast_fade_overlap_prop: draw_config.get_f64(&PgConfigKey::FadeOverlap(CarriageSpeed::Quick))?,
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
                return Err(Message::CodeInvariantFailed("overlapping fades sent to UI".to_string()));
            }
        };
        self.fade_state = FadeState::Fading(from,index,speed,None,Arc::new(self.redraw_needed.clone().lock()));
        Ok(())
    }

    fn prop(&self, speed: &CarriageSpeed, elapsed: f64) -> f64 {
        let fade_time = match speed {
            CarriageSpeed::Quick => self.fast_fade_time,
            CarriageSpeed::SlowCrossFade => self.slow_cross_fade_time,
            CarriageSpeed::Slow => self.slow_fade_time
        };
        elapsed/fade_time
    }

    fn fade_time(&self, speed: &CarriageSpeed, elapsed: f64, out: bool) -> f64 {
        let factor = match speed {
            CarriageSpeed::Quick => self.fast_fade_overlap_prop,
            CarriageSpeed::SlowCrossFade => self.slow_cross_fade_overlap_prop,
            CarriageSpeed::Slow => self.slow_fade_overlap_prop
        };
        let prop = self.prop(speed,elapsed).min(1.).max(0.)*(1.+factor.abs());
        let val = match (factor>0.,out) {
            (true,true) => { 1.-prop }, /* out before in; out */
            (true,false) => { prop-factor }, /* out before in; in */
            (false,true) => { 1.-(prop+factor) }, /* in before out; out */
            (false,false) => { prop } /* in before out; in */
        }.min(1.).max(0.);
        val
    }

    fn notify_fade_state(&mut self,gl: &WebGlGlobal) {
        match self.fade_state.clone() {
            FadeState::Constant(None) => {},
            FadeState::Constant(Some(index)) => {
                self.get_train(gl,index).set_opacity(1.);
            },
            FadeState::Fading(from,to,speed,Some(elapsed),_) => {
                let prop_out = self.fade_time(&speed,elapsed,true);
                let prop_in = self.fade_time(&speed,elapsed,false);
                self.get_train(gl,to).set_opacity(prop_in);
                if let Some(from) = from {
                    self.get_train(gl,from).set_opacity(prop_out);
                }
            },
            FadeState::Fading(from,to,speed,None,_) => {}
        }
    }

    fn transition_animate_tick(&mut self, gl: &mut WebGlGlobal, newly_elapsed: f64) -> Result<bool,Message> {
        let mut complete = false;
        match self.fade_state.clone() {
            FadeState::Constant(_) => {}
            FadeState::Fading(from,to,speed,mut elapsed,redraw) => {
                elapsed = Some(elapsed.map(|e| e+newly_elapsed).unwrap_or(0.));
                let prop = self.prop(&speed,elapsed.unwrap());
                if prop >= 1.{
                    if let Some(from) = from {
                        self.get_train(gl,from).discard(gl)?;
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

    fn draw_animate_tick(&mut self, stage: &ReadStage, gl: &mut WebGlGlobal, session: &DrawingSession) -> Result<(),Message> {
        match self.fade_state.clone() {
            FadeState::Constant(None) => {},
            FadeState::Constant(Some(train)) => {
                self.get_train(gl,train).draw(gl,stage,&session)?;
            },
            FadeState::Fading(from,to,_,_,_) => {
                if let Some(from) = from {
                    self.get_train(gl,from).draw(gl,stage,&session)?;
                }
                self.get_train(gl,to).draw(gl,stage,&session)?;
            },
        }
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
    pub fn new(draw_config: &PgPeregrineConfig, stage: &Stage) -> Result<GlTrainSet,Message> {
        Ok(GlTrainSet {
            data: Arc::new(Mutex::new(GlTrainSetData::new(draw_config,&stage.redraw_needed())?))
        })
    }

    pub fn transition_animate_tick(&mut self, api: &PeregrineCore, gl: &mut WebGlGlobal, newly_elapsed: f64) -> Result<(),Message> {
        if self.data.lock().unwrap().transition_animate_tick(gl,newly_elapsed)? {
            api.transition_complete();
        }
        Ok(())
    }

    pub(crate) fn draw_animate_tick(&mut self, stage: &ReadStage, gl: &mut WebGlGlobal, session: &DrawingSession) -> Result<(),Message> {
        self.data.lock().unwrap().draw_animate_tick(stage,gl,session)
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