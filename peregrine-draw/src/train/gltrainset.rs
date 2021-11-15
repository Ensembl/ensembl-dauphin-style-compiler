use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{ Arc, Mutex };
use peregrine_data::{Assets, Carriage, CarriageSpeed, PeregrineCore, Scale, ZMenuProxy};
use peregrine_toolkit::lock;
use peregrine_toolkit::sync::needed::{Needed, NeededLock};
use super::gltrain::GLTrain;
use crate::{run::{ PgPeregrineConfig, PgConfigKey }, stage::stage::{ Stage, ReadStage } };
use crate::webgl::DrawingSession;
use crate::webgl::global::WebGlGlobal;
use crate::util::message::Message;
use web_sys::console;

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

    fn get_train(&mut self, index: u32) -> &mut GLTrain {
        if !self.trains.contains_key(&index) {
            self.trains.insert(index,GLTrain::new(&self.redraw_needed));
        }
        self.trains.get_mut(&index).unwrap()
    }

    fn set_carriages(&mut self, gl: &mut WebGlGlobal, assets: &Assets, new_carriages: &[Carriage], scale: &Scale, index: u32) -> Result<(),Message> {
        self.get_train(index).set_carriages(scale,new_carriages,gl,assets)
    }

    fn set_max(&mut self,index: u32, len: u64) {
        self.get_train(index).set_max(len);
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

    fn notify_fade_state(&mut self) {
        match self.fade_state.clone() {
            FadeState::Constant(None) => {},
            FadeState::Constant(Some(index)) => {
                self.get_train(index).set_opacity(1.);
            },
            FadeState::Fading(from,to,speed,Some(elapsed),_) => {
                let prop_out = self.fade_time(&speed,elapsed,true);
                let prop_in = self.fade_time(&speed,elapsed,false);
                self.get_train(to).set_opacity(prop_in);
                if let Some(from) = from {
                    self.get_train(from).set_opacity(prop_out);
                }
            },
            FadeState::Fading(_,_,_,None,_) => {}
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
                        self.get_train(from).discard(gl)?;
                        self.trains.remove(&from);
                    }
                    self.fade_state = FadeState::Constant(Some(to));
                    self.redraw_needed.set(); // probably not needed; belt-and-braces
                    complete = true;
                } else {
                    self.fade_state = FadeState::Fading(from,to,speed.clone(),elapsed,redraw);
                }
                self.notify_fade_state();
            }
        }
        Ok(complete)
    }

    fn train_scale(&mut self, index: u32)-> u64 {
        self.get_train(index).scale().map(|x| x.get_index()).unwrap_or(0)
    }

    fn draw_animate_tick(&mut self, stage: &ReadStage, gl: &mut WebGlGlobal, session: &mut DrawingSession) -> Result<(),Message> {
        match self.fade_state.clone() {
            FadeState::Constant(None) => {},
            FadeState::Constant(Some(train)) => {
                self.get_train(train).draw(gl,stage,session)?;
            },
            FadeState::Fading(from,to,_,_,_) => {
                if let Some(from) = from {
                    if self.train_scale(from) > self.train_scale(to) {
                        /* zooming in, give priority to more detailed target */
                        self.get_train(to).draw(gl,stage,session)?;
                        self.get_train(from).draw(gl,stage,session)?;
                    } else {
                        /* zooming out, give priority to more detailed source */
                        self.get_train(from).draw(gl,stage,session)?;
                        self.get_train(to).draw(gl,stage,session)?;
                    }
                } else {
                    self.get_train(to).draw(gl,stage,session)?;
                }
            },
        }
        Ok(())
    }

    fn scale(&mut self) -> Option<Scale> {
        match self.fade_state.clone() {
            FadeState::Constant(None) => None,
            FadeState::Constant(Some(train)) => self.get_train(train).scale(),
            FadeState::Fading(_,to,_,_,_) => self.get_train(to).scale()
        }
    }

    fn get_hotspot(&mut self, stage: &ReadStage, position: (f64,f64)) -> Result<Vec<Rc<ZMenuProxy>>,Message> {
        match self.fade_state {
            FadeState::Constant(x) => x,
            FadeState::Fading(_,x,_,_,_) => Some(x)
        }.map(|id| {
            self.get_train(id).get_hotspot(stage,position)
        }).unwrap_or(Ok(vec![]))
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

    pub(crate) fn draw_animate_tick(&mut self, stage: &ReadStage, gl: &mut WebGlGlobal, session: &mut DrawingSession) -> Result<(),Message> {
        self.data.lock().unwrap().draw_animate_tick(stage,gl,session)
    }

    pub fn set_carriages(&mut self, new_carriages: &[Carriage], scale: &Scale, gl: &mut WebGlGlobal, assets: &Assets, index: u32) -> Result<(),Message> {
        self.data.lock().unwrap().set_carriages(gl,assets,new_carriages,scale,index)
    }

    pub fn start_fade(&mut self, index: u32, max: u64, speed: CarriageSpeed) -> Result<(),Message> {
        self.data.lock().unwrap().start_fade(index,speed)?;
        self.data.lock().unwrap().set_max(index,max);
        Ok(())
    }

    pub(crate) fn get_hotspot(&self,stage: &ReadStage, position: (f64,f64)) -> Result<Vec<Rc<ZMenuProxy>>,Message> {
        lock!(self.data).get_hotspot(stage,position)
    }

    pub fn discard(&mut self, gl: &mut WebGlGlobal) -> Result<(),Message> {
        lock!(self.data).discard(gl)
    }

    pub fn scale(&self) -> Option<Scale> { lock!(self.data).scale() }
}