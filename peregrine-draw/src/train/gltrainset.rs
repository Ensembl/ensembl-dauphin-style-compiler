use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash};
use std::sync::{ Arc, Mutex };
use peregrine_data::{Assets, CarriageSpeed, PeregrineCore, Scale, DrawingCarriage, TrainIdentity, PeregrineApiQueue, SingleHotspotEntry, SpecialClick};
use peregrine_toolkit::error::Error;
use peregrine_toolkit::{lock};
use peregrine_toolkit_async::sync::needed::{Needed, NeededLock};
use super::glcarriage::GLCarriage;
use super::gltrain::GLTrain;
use crate::{PgCommanderWeb};
use crate::{run::{ PgPeregrineConfig, PgConfigKey }, stage::stage::{ Stage, ReadStage } };
use crate::webgl::DrawingSession;
use crate::webgl::global::WebGlGlobal;
use crate::util::message::Message;

#[cfg(debug_trains)]
use peregrine_toolkit::{log, debug_log};

#[derive(Clone)]
enum FadeState {
    Constant(Option<TrainIdentity>),
    Fading(Option<TrainIdentity>,TrainIdentity,CarriageSpeed,Option<f64>,Arc<NeededLock>)
}

struct GlRailwayData {
    slow_fade_time: f64,
    slow_cross_fade_time: f64,
    fast_fade_time: f64,
    slow_fade_overlap_prop: f64,
    slow_cross_fade_overlap_prop: f64,
    fast_fade_overlap_prop: f64,
    data_api: PeregrineApiQueue,
    commander: PgCommanderWeb,
    trains: HashMap<TrainIdentity,GLTrain>,
    carriages: HashMap<DrawingCarriage,GLCarriage>,
    fade_state: FadeState,
    redraw_needed: Needed
}

impl GlRailwayData {
    fn new(data_api: &PeregrineApiQueue, commander: &PgCommanderWeb, draw_config: &PgPeregrineConfig,redraw_needed: &Needed) -> Result<GlRailwayData,Message> {
        Ok(GlRailwayData {
            commander: commander.clone(),
            slow_fade_time: draw_config.get_f64(&PgConfigKey::AnimationFadeRate(CarriageSpeed::Slow))?,
            slow_cross_fade_time: draw_config.get_f64(&PgConfigKey::AnimationFadeRate(CarriageSpeed::SlowCrossFade))?,
            fast_fade_time: draw_config.get_f64(&PgConfigKey::AnimationFadeRate(CarriageSpeed::Quick))?,
            slow_fade_overlap_prop: draw_config.get_f64(&PgConfigKey::FadeOverlap(CarriageSpeed::Slow))?,
            slow_cross_fade_overlap_prop: draw_config.get_f64(&PgConfigKey::FadeOverlap(CarriageSpeed::SlowCrossFade))?,
            fast_fade_overlap_prop: draw_config.get_f64(&PgConfigKey::FadeOverlap(CarriageSpeed::Quick))?,
            trains: HashMap::new(),
            carriages: HashMap::new(),
            fade_state: FadeState::Constant(None),
            redraw_needed: redraw_needed.clone(),
            data_api: data_api.clone(),
        })
    }

    fn get_our_train(&mut self, identity: &TrainIdentity, id: u64) -> &mut GLTrain {
        self.trains.get_mut(identity).expect(&format!("no self train {}",id))
    }

    fn create_train(&mut self, extent: &TrainIdentity) {
        #[cfg(debug_trains)] log!("GL create train {:?}",extent);
        self.trains.insert(extent.clone(),GLTrain::new(&self.redraw_needed));
    }

    #[cfg(any(debug_assertions,debug_trains))] 
    fn check_train_unused_one(&self, a: Option<&TrainIdentity>, b: &TrainIdentity) {
        if let Some(a) = a {
            if a == b {
                panic!("disposing of train in use! {:?}",b);
            }
        }
    }

    #[cfg(any(debug_assertions,debug_trains))] 
    fn check_train_unused(&self, extent: &TrainIdentity) {
        match &self.fade_state {
            FadeState::Constant(id) => {
                self.check_train_unused_one(id.as_ref(),extent);
            },
            FadeState::Fading(from, to, _, _, _) => {
                self.check_train_unused_one(from.as_ref(),extent);
                self.check_train_unused_one(Some(to),extent);
            }
        }
    }

    fn drop_train(&mut self, extent: &TrainIdentity) {
        #[cfg(any(debug_assertions,debug_trains))] 
        self.check_train_unused(extent);
        #[cfg(debug_trains)] log!("GL drop train {:?}",extent);
        self.trains.remove(extent);
    }

    fn create_carriage(&mut self, carriage: &DrawingCarriage, gl: &Arc<Mutex<WebGlGlobal>>, assets: &Assets) -> Result<(),Message> {
        if !self.carriages.contains_key(&carriage) {
            self.carriages.insert(carriage.clone(), GLCarriage::new(&self.data_api,&self.redraw_needed,&self.commander,carriage, gl, assets)?);
        }
        Ok(())
    }

    fn drop_carriage(&mut self, carriage: &DrawingCarriage) { 
        self.carriages.remove(carriage);
    }

    fn set_carriages(&mut self, extent: &TrainIdentity, new_carriages: &[DrawingCarriage]) -> Result<(),Message> {
        match new_carriages.iter().map(|c| self.carriages.get(c).cloned()).collect::<Option<Vec<_>>>() {
            Some(carriages) => {
                let mut hash = DefaultHasher::new();
                extent.hash(&mut hash);
                let out = self.get_our_train(&extent,1).set_carriages(carriages);
                self.redraw_needed.set();
                out
            },
            None => {
                Err(Message::CodeInvariantFailed(format!("missing carriages")))
            }
        }
    }

    fn set_max(&mut self, extent: &TrainIdentity, len: u64) {
        self.get_our_train(extent,2).set_max(len);
    }

    fn start_fade(&mut self, train: &TrainIdentity, speed: CarriageSpeed) -> Result<(),Error> {
        let from = match &self.fade_state {            
            FadeState::Constant(x) => x,
            FadeState::Fading(_,_,_,_,_) => {
                return Err(Error::fatal("overlapping fades sent to UI"));
            }
        };
        #[cfg(debug_trains)]
        debug_log!("fading start {:?}",speed);
        self.fade_state = FadeState::Fading(from.clone(),train.clone(),speed,None,Arc::new(self.redraw_needed.clone().lock()));
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
            (true,true) => { 1.-prop },           /* fade-out then fade-in ; fade-out opacity */
            (true,false) => { prop-factor },      /* fade-out then fade-in ; fade-in  opacity */
            (false,true) => { 1.-(prop+factor) }, /* fade-in  then fade-out; fade-out opacity */
            (false,false) => { prop }             /* fase-in  then fade-out; fade-in  opacity */
        }.min(1.).max(0.);
        val
    }

    fn notify_fade_state(&mut self) {
        match self.fade_state.clone() {
            FadeState::Constant(None) => {},
            FadeState::Constant(Some(extent)) => {
                self.get_our_train(&extent,3).set_opacity(1.);
            },
            FadeState::Fading(from,to,speed,Some(elapsed),_) => {
                let prop_out = self.fade_time(&speed,elapsed,true);
                let prop_in = self.fade_time(&speed,elapsed,false);
                self.get_our_train(&to,4).set_opacity(prop_in);
                if let Some(from) = from {
                    self.get_our_train(&from,5).set_opacity(prop_out);
                }
            },
            FadeState::Fading(_,_,_,None,_) => {}
        }
    }

    fn transition_animate_tick(&mut self, _gl: &mut WebGlGlobal, newly_elapsed: f64) -> Result<bool,Message> {
        let mut complete = false;
        match self.fade_state.clone() {
            FadeState::Constant(_) => {}
            FadeState::Fading(from,to,speed,mut elapsed,redraw) => {
                elapsed = Some(elapsed.map(|e| e+newly_elapsed).unwrap_or(0.));
                let prop = self.prop(&speed,elapsed.unwrap());
                if prop >= 1. {
                    #[cfg(debug_trains)]
                    debug_log!("fading done {:?}",from);
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

    fn train_scale(&mut self, extent: &TrainIdentity)-> u64 {
        self.get_our_train(extent,6).scale().map(|x| x.get_index()).unwrap_or(0)
    }

    fn get_draws(&mut self) -> Vec<GLTrain> {
        let mut out = vec![];
        match self.fade_state.clone() {
            FadeState::Constant(None) => {},
            FadeState::Constant(Some(train)) => {
                out.push(self.get_our_train(&train,7).clone());
            },
            FadeState::Fading(from,to,_,_,_) => {
                if let Some(from) = from {
                    if self.train_scale(&from) > self.train_scale(&to) {
                        /* zooming in, give priority to more detailed target */
                        out.push(self.get_our_train(&to,8).clone());
                        out.push(self.get_our_train(&from,9).clone());
                    } else {
                        /* zooming out, give priority to more detailed source */
                        out.push(self.get_our_train(&from,10).clone());
                        out.push(self.get_our_train(&to,11).clone());                    }
                } else {
                    out.push(self.get_our_train(&to,12).clone());
                }
            },
        }
        out
    }

    fn scale(&mut self) -> Option<Scale> {
        match self.fade_state.clone() {
            FadeState::Constant(None) => None,
            FadeState::Constant(Some(train)) => self.get_our_train(&train,13).scale(),
            FadeState::Fading(_,to,_,_,_) => self.get_our_train(&to,14).scale()
        }
    }

    fn train_for_hotspots(&mut self) -> Option<&mut GLTrain> {
        match &self.fade_state {
            FadeState::Constant(x) => x.as_ref(),
            FadeState::Fading(_,x,_,_,_) => Some(x)
        }.cloned().as_ref().map(move |id| self.get_our_train(&id,15))
    }

    fn get_hotspot(&mut self, stage: &ReadStage, position: (f64,f64)) -> Result<Vec<SingleHotspotEntry>,Message> {
        self.train_for_hotspots().map(|t| {
            t.get_hotspot(stage,position)
        }).unwrap_or(Ok(vec![]))
    }

    fn any_hotspot(&mut self, stage: &ReadStage, position: (f64,f64)) -> Result<bool,Message> {
        self.train_for_hotspots().map(|t| {
            t.any_hotspot(stage,position)
        }).unwrap_or(Ok(false))
    }

    fn special_hotspots(&mut self, stage: &ReadStage, position: (f64,f64)) -> Result<Vec<SpecialClick>,Message> {
        self.train_for_hotspots().map(|t| {
            t.special_hotspots(stage,position)
        }).unwrap_or(Ok(vec![]))
    }
}

#[derive(Clone)]
pub struct GlRailway {
    data: Arc<Mutex<GlRailwayData>>
}

impl GlRailway {
    pub fn new(data_api: &PeregrineApiQueue, commander: &PgCommanderWeb, draw_config: &PgPeregrineConfig, stage: &Stage) -> Result<GlRailway,Message> {
        Ok(GlRailway {
            data: Arc::new(Mutex::new(GlRailwayData::new(data_api,commander,draw_config,&stage.redraw_needed())?))
        })
    }

    pub fn create_train(&mut self, train: &TrainIdentity) { lock!(self.data).create_train(train) }
    pub fn drop_train(&mut self, train: &TrainIdentity) { lock!(self.data).drop_train(train) }

    pub(crate) fn create_carriage(&mut self, carriage: &DrawingCarriage, gl: &Arc<Mutex<WebGlGlobal>>, assets: &Assets) -> Result<(),Message> {
        lock!(self.data).create_carriage(carriage,gl,assets)
    }

    pub(crate) fn drop_carriage(&mut self, carriage: &DrawingCarriage) { lock!(self.data).drop_carriage(carriage); }

    pub fn transition_animate_tick(&mut self, api: &PeregrineCore, gl: &mut WebGlGlobal, newly_elapsed: f64) -> Result<(),Message> {
        if lock!(self.data).transition_animate_tick(gl,newly_elapsed)? {
            api.transition_complete();
        }
        Ok(())
    }

    pub(crate) fn draw_animate_tick(&mut self, stage: &ReadStage, gl: &Arc<Mutex<WebGlGlobal>>, session: &mut DrawingSession) -> Result<(),Message> {
        let mut state =  lock!(self.data);
        let mut draws = state.get_draws();
        drop(state);
        for mut train in draws.drain(..) {
            train.draw(gl,stage,session)?;
        }
        Ok(())
    }

    pub fn set_carriages(&mut self, train: &TrainIdentity, new_carriages: &[DrawingCarriage]) -> Result<(),Message> {
        lock!(self.data).set_carriages(train,new_carriages)?;
        Ok(())
    }

    pub fn start_fade(&mut self, train: &TrainIdentity, max: u64, speed: CarriageSpeed) -> Result<(),Error> {
        self.data.lock().unwrap().start_fade(train,speed)?;
        self.data.lock().unwrap().set_max(&train,max);
        Ok(())
    }

    pub(crate) fn get_hotspot(&self,stage: &ReadStage, position: (f64,f64)) -> Result<Vec<SingleHotspotEntry>,Message> {
        lock!(self.data).get_hotspot(stage,position)
    }

    pub(crate) fn any_hotspot(&self,stage: &ReadStage, position: (f64,f64)) -> Result<bool,Message> {
        lock!(self.data).any_hotspot(stage,position)
    }

    pub(crate) fn special_hotspots(&self,stage: &ReadStage, position: (f64,f64)) -> Result<Vec<SpecialClick>,Message> {
        lock!(self.data).special_hotspots(stage,position)
    }

    pub fn scale(&self) -> Option<Scale> { lock!(self.data).scale() }
}
