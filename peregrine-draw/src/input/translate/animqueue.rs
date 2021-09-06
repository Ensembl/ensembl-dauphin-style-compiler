use std::collections::VecDeque;
use crate::run::{PgConfigKey, PgPeregrineConfig};
use crate::run::report::Report;
use crate::{Message, PeregrineInnerAPI };
use super::axisphysics::{AxisPhysicsConfig, Scaling};
use super::dragregime::{PhysicsDragRegimeCreator, PhysicsRunnerDragRegime};
use super::measure::Measure;
use super::windowregime::{PhysicsRunnerWRegime, PhysicsWRegimeCreator};

pub(super) enum Cadence {
    #[allow(unused)]
    UserInput,
    Instructed,
    SelfPropelled
}

pub(super) enum QueueEntry {
    MoveW(f64,f64),
    ShiftTo(f64,Cadence),
    ZoomTo(f64,Cadence),
    ShiftMore(f64),
    ZoomMore(f64,Option<f64>),
    BrakeX,
    BrakeZ,
    Wait,
    Size(f64)
}

pub(super) enum ApplyResult {
    Finished,
    Update(Option<f64>,Option<f64>)
}

macro_rules! set_regime {
    ($call:ident,$try_call:ident,$inner:ty,$branch:tt,$creator:tt) => {
        fn $call(&mut self, measure: &Measure) -> &mut $inner {
            let create = self.$try_call().is_none();
            if create {
                self.object = PhysicsRegimeObject::$branch(self.$creator.create());
                self.object.as_trait_mut().set_size(measure,self.size);
            }
            self.update_settings(measure);
            self.$try_call().unwrap()
        }

        #[allow(unused)]
        fn $try_call(&mut self) -> Option<&mut $inner> {
            match &mut self.object {
                PhysicsRegimeObject::$branch(out) => { return Some(out); },
                _ => { return None; }
            }
        }
    };
}

pub(super) trait PhysicsRegimeCreator {
    type Object;

    fn create(&self) -> Self::Object;
}

pub(super) trait PhysicsRegimeTrait {
    fn set_size(&mut self, measure: &Measure, size: Option<f64>);
    fn report_target(&mut self, measure: &Measure) -> (Option<f64>,Option<f64>);
    fn apply_spring(&mut self, measure: &Measure, total_dt: f64) -> ApplyResult;
    fn update_settings(&mut self, measure: &Measure);
    fn is_active(&self) -> bool { true }
}

struct PhysicsRegimeNone();

impl PhysicsRegimeTrait for PhysicsRegimeNone {
    fn set_size(&mut self, _measure: &Measure, _size: Option<f64>) {}
    fn report_target(&mut self, _measure: &Measure) -> (Option<f64>,Option<f64>) { (None,None) }
    fn apply_spring(&mut self, _measure: &Measure, _total_dt: f64) -> ApplyResult { ApplyResult::Finished }
    fn update_settings(&mut self, _measure: &Measure) {}
    fn is_active(&self) -> bool { false }
}

enum PhysicsRegimeObject {
    W(PhysicsRunnerWRegime),
    UserPull(PhysicsRunnerDragRegime),
    SelfPull(PhysicsRunnerDragRegime),
    None(PhysicsRegimeNone)
}

impl PhysicsRegimeObject {
    fn as_trait_mut(&mut self) -> &mut dyn PhysicsRegimeTrait {
        match self {
            PhysicsRegimeObject::W(r) => r,
            PhysicsRegimeObject::UserPull(r) => r,
            PhysicsRegimeObject::SelfPull(r) => r,
            PhysicsRegimeObject::None(r) => r
        }
    }
}

struct PhysicsRegime {
    object: PhysicsRegimeObject,
    w_creator: PhysicsWRegimeCreator,
    user_drag_creator: PhysicsDragRegimeCreator,
    instructed_drag_creator: PhysicsDragRegimeCreator,
    self_drag_creator: PhysicsDragRegimeCreator,
    size: Option<f64>
}

fn make_axis_config(config: &PgPeregrineConfig, lethargy_key: &PgConfigKey) -> Result<AxisPhysicsConfig,Message> {
    Ok(AxisPhysicsConfig {
        lethargy: config.get_f64(lethargy_key)?,
        boing: config.get_f64(&PgConfigKey::AnimationBoing)?,
        vel_min: config.get_f64(&PgConfigKey::AnimationVelocityMin)?,
        force_min: config.get_f64(&PgConfigKey::AnimationForceMin)?,
        brake_mul: config.get_f64(&PgConfigKey::AnimationBrakeMul)?,
        min_bp_per_screen: config.get_f64(&PgConfigKey::MinBpPerScreen)?,
        scaling: Scaling::Linear(1.)
    })
}

fn make_drag_axis_config(config: &PgPeregrineConfig, lethargy_key: &PgConfigKey) -> Result<(AxisPhysicsConfig,AxisPhysicsConfig),Message> {
    let x_config = make_axis_config(config,lethargy_key)?;
    let mut z_config = make_axis_config(config,lethargy_key)?;
    z_config.scaling = Scaling::Logarithmic(100.);
    Ok((x_config,z_config))
}

impl PhysicsRegime {
    fn new(config: &PgPeregrineConfig) -> Result<PhysicsRegime,Message> {
        let user_drag_config = make_drag_axis_config(config,&PgConfigKey::UserDragLethargy)?;
        let instructed_drag_config = make_drag_axis_config(config,&PgConfigKey::InstructedDragLethargy)?;
        let self_drag_config = make_drag_axis_config(config,&PgConfigKey::SelfDragLethargy)?;
        let w_config = make_axis_config(config,&PgConfigKey::WindowLethargy)?;
        Ok(PhysicsRegime {
            object: PhysicsRegimeObject::None(PhysicsRegimeNone()),
            w_creator: PhysicsWRegimeCreator(w_config),
            user_drag_creator: PhysicsDragRegimeCreator(user_drag_config.0,user_drag_config.1),
            instructed_drag_creator: PhysicsDragRegimeCreator(instructed_drag_config.0,instructed_drag_config.1),
            self_drag_creator: PhysicsDragRegimeCreator(self_drag_config.0,self_drag_config.1),
            size: None
        })
    }

    set_regime!(regime_w,try_regime_w,PhysicsRunnerWRegime,W,w_creator);
    set_regime!(regime_user_drag,try_regime_user_drag,PhysicsRunnerDragRegime,UserPull,user_drag_creator);
    set_regime!(regime_instructed_drag,try_regime_instructed_drag,PhysicsRunnerDragRegime,UserPull,instructed_drag_creator);
    set_regime!(regime_self_drag,try_regime_self_drag,PhysicsRunnerDragRegime,SelfPull,self_drag_creator);

    fn apply_spring(&mut self, measure: &Measure, total_dt: f64) -> (Option<f64>,Option<f64>) {
        match self.object.as_trait_mut().apply_spring(measure,total_dt) {
            ApplyResult::Update(x,bp) => (x,bp),
            ApplyResult::Finished => {
                self.object = PhysicsRegimeObject::None(PhysicsRegimeNone());
                (None,None)
            }
        }
    }

    fn is_active(&mut self) -> bool {
        self.object.as_trait_mut().is_active()
    }

    fn report_target(&mut self, measure: &Measure) -> (Option<f64>,Option<f64>) {
        self.object.as_trait_mut().report_target(measure)
    }

    fn update_settings(&mut self, measure: &Measure) {
        self.object.as_trait_mut().update_settings(measure);
    }

    fn set_size(&mut self, measure: &Measure, size: f64) {
        if let Some(old_size) = self.size {
            if old_size == size { return; }
        }
        self.size = Some(size);
        self.object.as_trait_mut().set_size(measure,self.size);
    }
}

pub(super) struct PhysicsRunner {
    regime: PhysicsRegime,
    animation_queue: VecDeque<QueueEntry>,
    animation_current: Option<QueueEntry>
}

impl PhysicsRunner {
    pub(super) fn new(config: &PgPeregrineConfig) -> Result<PhysicsRunner,Message> {
        Ok(PhysicsRunner {
            regime: PhysicsRegime::new(config)?,
            animation_queue: VecDeque::new(),
            animation_current: None,
        })
    }

    pub(super) fn queue_clear(&mut self) {
        self.animation_queue.clear();
    }

    pub(super) fn queue_add(&mut self, entry: QueueEntry) {
        self.animation_queue.push_back(entry);
    }

    pub(super) fn update_needed(&mut self) -> bool{
        self.regime.is_active() || self.animation_queue.len() !=0 || self.animation_current.is_some()
    }

    pub(super) fn apply_spring(&mut self, inner: &mut PeregrineInnerAPI, total_dt: f64) -> Result<(),Message> {
        let measure = if let Some(measure) = Measure::new(inner)? { measure } else { return Ok(()); };
        self.regime.update_settings(&measure);
        let (new_x,new_bp) = self.regime.apply_spring(&measure,total_dt);
        if let Some(new_x) = new_x {
            inner.set_x(new_x);
        }
        if let Some(bp_per_screen) = new_bp {
            inner.set_bp_per_screen(bp_per_screen);
        }
        Ok(())
    }

    fn run_one_step(&mut self, measure: &Measure, entry: &QueueEntry) {
        self.regime.update_settings(measure);
        match &entry {
            QueueEntry::Wait => {},
            QueueEntry::MoveW(centre,scale) => {
                self.regime.regime_w(measure).set(measure,*centre,*scale);
            },
            QueueEntry::ShiftTo(amt,cadence) => {
                match cadence {
                    Cadence::UserInput => { self.regime.regime_user_drag(measure).shift_to(*amt); },
                    Cadence::Instructed => { self.regime.regime_instructed_drag(measure).shift_to(*amt); },
                    Cadence::SelfPropelled => { self.regime.regime_self_drag(measure).shift_to(*amt); }
                }
            },
            QueueEntry::ZoomTo(amt,cadence) => {
                match cadence {
                    Cadence::UserInput => { self.regime.regime_user_drag(measure).zoom_to(*amt); },
                    Cadence::Instructed => { self.regime.regime_instructed_drag(measure).zoom_to(*amt); },
                    Cadence::SelfPropelled => { self.regime.regime_self_drag(measure).zoom_to(*amt); }
                }
            },
            QueueEntry::ShiftMore(amt) => {
                self.regime.regime_user_drag(measure).shift_more(&measure,*amt);
            },
            QueueEntry::ZoomMore(amt,pos) => { 
                self.regime.regime_user_drag(measure).zoom_more(&measure,*amt,pos.clone());
            },
            QueueEntry::BrakeX => {
                if let Some(drag) = self.regime.try_regime_user_drag() { drag.brake_x(); }
                if let Some(drag) = self.regime.try_regime_self_drag() { drag.brake_x(); }
            },
            QueueEntry::BrakeZ => { 
                if let Some(drag) = self.regime.try_regime_user_drag() { drag.brake_z(); }
                if let Some(drag) = self.regime.try_regime_self_drag() { drag.brake_z(); }
            },
            QueueEntry::Size(size) => {
                self.regime.set_size(measure,*size);
            }
        }
    }

    fn report_targets(&mut self, measure: &Measure, report: &mut Report) {
        let (target_x,target_bp) = self.regime.report_target(&measure);
        if let Some(target_x) = target_x {
            report.set_target_x_bp(target_x);
        }
        if let Some(target_bp) = target_bp {
            report.set_target_bp_per_screen(target_bp);
        }
    }

    fn exit_due_to_waiting(&mut self) -> bool {
        if let Some(entry) = self.animation_queue.front() {
            match entry {
                QueueEntry::Wait => {
                    if self.regime.is_active() { return true; }
                },
                _ => {}
            }
        }
        false
    }

    pub(super) fn drain_animation_queue(&mut self, inner: &PeregrineInnerAPI, report: &mut Report) -> Result<(),Message> {
        loop {
            if self.exit_due_to_waiting() { break; }
            self.animation_current = self.animation_queue.pop_front();
            if self.animation_current.is_none() { break; }
            /* do it */
            let current = self.animation_current.take();
            let measure = if let Some(measure) = Measure::new(inner)? { measure } else { return Ok(()); };
            if let Some(entry) = &current {
                self.run_one_step(&measure,entry);
            }
            self.report_targets(&measure,report);
            self.animation_current = current;
        }
        Ok(())
    }
}
