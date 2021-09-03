use std::collections::VecDeque;
use crate::run::report::Report;
use crate::{Message, PeregrineInnerAPI };
use super::dragregime::PhysicsRunnerDragRegime;
use super::measure::Measure;
use super::windowregime::PhysicsRunnerWRegime;

pub(super) fn bp_to_zpx(bp: f64) -> f64 { bp.log2() * 100. }

pub(super) enum QueueEntry {
    MoveW(f64,f64),
    MoveX(f64),
    MoveZ(f64,Option<f64>),
    JumpX(f64),
    JumpZ(f64,Option<f64>),
    BrakeX,
    BrakeZ,
    Wait
}

pub(super) enum ApplyResult {
    Finished,
    Update(Option<f64>,Option<f64>)
}

macro_rules! set_regime {
    ($call:ident,$try_call:ident,$inner:ty,$branch:tt,$ctor:expr) => {
        fn $call(&mut self) -> &mut $inner {
            match self {
                PhysicsRegime::$branch(out) => { return out; },
                _ => {}
            }
            *self = PhysicsRegime::$branch($ctor);
            match self {
                PhysicsRegime::$branch(out) => { return out; },
                _ => { panic!("impossible regime create") }
            }
        }

        #[allow(unused)]
        fn $try_call(&mut self) -> Option<&mut $inner> {
            match self {
                PhysicsRegime::$branch(out) => { return Some(out); },
                _ => { return None; }
            }
        }
    };
}

enum PhysicsRegime {
    W(PhysicsRunnerWRegime),
    Pull(PhysicsRunnerDragRegime),
    None
}

impl PhysicsRegime {
    fn is_active(&self) -> bool {
        match self {
            PhysicsRegime::None => false,
            _ => true
        }
    }

    set_regime!(regime_w,try_regime_w,PhysicsRunnerWRegime,W,PhysicsRunnerWRegime::new());
    set_regime!(regime_drag,try_regime_drag,PhysicsRunnerDragRegime,Pull,PhysicsRunnerDragRegime::new());

    fn apply_spring(&mut self, measure: &Measure, total_dt: f64) -> (Option<f64>,Option<f64>) {
        let result = match self {
            PhysicsRegime::W(r) => r.apply_spring(measure,total_dt),
            PhysicsRegime::Pull(r) => r.apply_spring(measure,total_dt),
            PhysicsRegime::None => ApplyResult::Finished
        };
        match result {
            ApplyResult::Update(x,bp) => (x,bp),
            ApplyResult::Finished => {
                *self = PhysicsRegime::None;
                (None,None)
            }
        }
    }

    fn report_target(&self, measure: &Measure) -> (Option<f64>,Option<f64>) {
        match self {
            PhysicsRegime::W(r) => r.report_target(measure),
            PhysicsRegime::Pull(r) => r.report_target(measure),
            PhysicsRegime::None => (None,None)
        }
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
            regime: PhysicsRegime::None,
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
        match &entry {
            QueueEntry::Wait => {},
            QueueEntry::MoveW(centre,scale) => {
                self.regime.regime_w().set(measure,*centre,*scale);
            },
            QueueEntry::MoveX(amt) => {
                self.regime.regime_drag().jump_x(measure,*amt);
            },
            QueueEntry::MoveZ(amt,centre) => {
                let amt = bp_to_zpx(*amt);
                self.regime.regime_drag().jump_z(measure,amt,centre.clone());
            },
            QueueEntry::JumpX(amt) => {
                self.regime.regime_drag().move_x(&measure,*amt);
            },
            QueueEntry::JumpZ(amt,pos) => { 
                self.regime.regime_drag().move_z(&measure,*amt,pos.clone());
            },
            QueueEntry::BrakeX => {
                if let Some(drag) = self.regime.try_regime_drag() { drag.brake_x(); }
            },
            QueueEntry::BrakeZ => { 
                if let Some(drag) = self.regime.try_regime_drag() { drag.brake_z(); }
            }
        }
    }

    fn report_targets(&self, measure: &Measure, report: &mut Report) {
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
