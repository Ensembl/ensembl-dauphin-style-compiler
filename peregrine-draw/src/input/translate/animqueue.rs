use std::collections::VecDeque;
use crate::run::report::Report;
use crate::{Message, PeregrineInnerAPI };
use super::dragregime::PhysicsRunnerDragRegime;
use super::measure::Measure;
use super::windowregime::PhysicsRunnerWRegime;

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
    ($call:ident,$try_call:ident,$inner:ty,$branch:tt,$ctor:ty,$params:expr) => {
        fn $call(&mut self, measure: &Measure) -> &mut $inner {
            let create = self.$try_call().is_none();
            if create {
                self.object = PhysicsRegimeObject::$branch(<$ctor>::new($params));
                self.object.set_size(measure,self.size);
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

enum PhysicsRegimeObject {
    W(PhysicsRunnerWRegime),
    UserPull(PhysicsRunnerDragRegime),
    SelfPull(PhysicsRunnerDragRegime),
    None
}

impl PhysicsRegimeObject {
    fn set_size(&mut self, measure: &Measure, size: Option<f64>) {
        match self {
            PhysicsRegimeObject::W(r) => r.set_size(measure,size),
            PhysicsRegimeObject::UserPull(r) => r.set_size(measure,size),
            PhysicsRegimeObject::SelfPull(r) => r.set_size(measure,size),
            PhysicsRegimeObject::None => {}
        }
    }
}

struct PhysicsRegime {
    object: PhysicsRegimeObject,
    size: Option<f64>
}

impl PhysicsRegime {
    fn new() -> PhysicsRegime {
        PhysicsRegime {
            object: PhysicsRegimeObject::None,
            size: None
        }
    }

    fn is_active(&self) -> bool {
        match self.object {
            PhysicsRegimeObject::None => false,
            _ => true
        }
    }

    set_regime!(regime_w,try_regime_w,PhysicsRunnerWRegime,W,PhysicsRunnerWRegime,());
    set_regime!(regime_user_drag,try_regime_user_drag,PhysicsRunnerDragRegime,UserPull,PhysicsRunnerDragRegime,500.);
    set_regime!(regime_instructed_drag,try_regime_instructed_drag,PhysicsRunnerDragRegime,UserPull,PhysicsRunnerDragRegime,1000.);
    set_regime!(regime_self_drag,try_regime_self_drag,PhysicsRunnerDragRegime,SelfPull,PhysicsRunnerDragRegime,25000.);

    fn apply_spring(&mut self, measure: &Measure, total_dt: f64) -> (Option<f64>,Option<f64>) {
        let result = match &mut self.object {
            PhysicsRegimeObject::W(r) => r.apply_spring(measure,total_dt),
            PhysicsRegimeObject::UserPull(r) => r.apply_spring(measure,total_dt),
            PhysicsRegimeObject::SelfPull(r) => r.apply_spring(measure,total_dt),
            PhysicsRegimeObject::None => ApplyResult::Finished
        };
        match result {
            ApplyResult::Update(x,bp) => (x,bp),
            ApplyResult::Finished => {
                self.object = PhysicsRegimeObject::None;
                (None,None)
            }
        }
    }

    fn report_target(&mut self, measure: &Measure) -> (Option<f64>,Option<f64>) {
        match &mut self.object {
            PhysicsRegimeObject::W(r) => r.report_target(measure),
            PhysicsRegimeObject::UserPull(r) => r.report_target(measure),
            PhysicsRegimeObject::SelfPull(r) => r.report_target(measure),
            PhysicsRegimeObject::None => (None,None)
        }
    }

    fn update_settings(&mut self, measure: &Measure) {
        match &mut self.object {
            PhysicsRegimeObject::W(r) => r.update_settings(measure),
            PhysicsRegimeObject::UserPull(r) => r.update_settings(measure),
            PhysicsRegimeObject::SelfPull(r) => r.update_settings(measure),
            PhysicsRegimeObject::None => {}
        }
    }

    fn set_size(&mut self, measure: &Measure, size: f64) {
        if let Some(old_size) = self.size {
            if old_size == size { return; }
        }
        self.size = Some(size);
        self.object.set_size(measure,self.size);
    }
}

pub(super) struct PhysicsRunner {
    regime: PhysicsRegime,
    animation_queue: VecDeque<QueueEntry>,
    animation_current: Option<QueueEntry>
}

impl PhysicsRunner {
    pub(super) fn new() -> PhysicsRunner {
        PhysicsRunner {
            regime: PhysicsRegime::new(),
            animation_queue: VecDeque::new(),
            animation_current: None,
        }
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

    fn exit_due_to_waiting(&self) -> bool {
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
