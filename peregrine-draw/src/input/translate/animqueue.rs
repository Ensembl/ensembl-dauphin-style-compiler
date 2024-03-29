use std::collections::VecDeque;
use peregrine_toolkit_async::sync::blocker::Lockout;

use crate::run::{PgConfigKey, PgPeregrineConfig};
use crate::run::report::Report;
use crate::util::message::Endstop;
use crate::{Message, PeregrineInnerAPI };
use super::measure::Measure;
use super::targetreporter::TargetReporter;
use crate::input::regimes::regime::Regime;

#[derive(Clone)]
pub(super) enum Cadence {
    Step,
    Smooth
}

#[derive(Debug)]
pub(super) enum QueueEntry {
    Set(f64,f64),
    Goto(Option<f64>,Option<f64>),
    ShiftMore(f64),
    ZoomMore(f64,Option<f64>),
    BrakeX,
    BrakeZ,
    Wait,
    Size(f64),
    Report,
    LockReports,
    Sketchy(bool)
}

pub(super) struct AnimationQueue {
    regime: Regime,
    animation_queue: VecDeque<QueueEntry>,
    intention_lockout: Option<Lockout>,
    animation_current: Option<QueueEntry>,
    size: Option<f64>,
    min_bp_per_screen: f64,
    target_reporter: TargetReporter,
}

impl AnimationQueue {
    pub(super) fn new(config: &PgPeregrineConfig, target_reporter: &TargetReporter) -> Result<AnimationQueue,Message> {
        Ok(AnimationQueue {
            regime: Regime::new(config)?,
            animation_queue: VecDeque::new(),
            intention_lockout: None,
            animation_current: None,
            size: None,
            min_bp_per_screen: config.get_f64(&PgConfigKey::MinBpPerScreen)?,
            target_reporter: target_reporter.clone()
        })
    }

    pub(super) fn remove_pending_actions(&mut self) {
        self.intention_lockout = None;
        let mut new_queue = VecDeque::new();
        for entry in self.animation_queue.drain(..) {
            match entry {
                QueueEntry::Size(_) |
                QueueEntry::Sketchy(_) |
                QueueEntry::Report |
                QueueEntry::LockReports => {
                    new_queue.push_back(entry);
                }
                _ => {}
            }
        }
        self.animation_queue = new_queue;
    }

    pub(super) fn queue_add(&mut self, entry: QueueEntry) {
        self.animation_queue.push_back(entry);
    }

    pub(super) fn update_needed(&mut self) -> bool{
        self.regime.is_active() || self.animation_queue.len() !=0 || self.animation_current.is_some()
    }

    pub(crate) fn regime_tick(&mut self, inner: &mut PeregrineInnerAPI, total_dt: f64) -> Result<bool,Message> {
        self.regime.tick(inner,total_dt)
    }

    fn run_one_queue_entry(&mut self, measure: &Measure, entry: &QueueEntry, inner: & PeregrineInnerAPI) {
        self.regime.update_settings(measure);
        match &entry {
            QueueEntry::LockReports => {
                self.intention_lockout = Some(self.target_reporter.lock_updates());
            }
            QueueEntry::Wait => {},
            QueueEntry::Set(centre,scale) => {
                self.regime.regime_set(measure).set(*centre,*scale);
            },
            QueueEntry::Goto(centre,scale) => {
                self.regime.regime_goto(measure).goto(*centre,*scale);
            },
            QueueEntry::ShiftMore(amt) => {
                self.regime.regime_user_drag(measure).shift_more(&measure,*amt);
            },
            QueueEntry::ZoomMore(amt,pos) => { 
                self.regime.regime_user_drag(measure).zoom_more(&measure,*amt,pos.clone());
            },
            QueueEntry::BrakeX => {
                if let Some(drag) = self.regime.try_regime_user_drag() { drag.brake_x(); }
            },
            QueueEntry::BrakeZ => { 
                if let Some(drag) = self.regime.try_regime_user_drag() { drag.brake_z(); }
            },
            QueueEntry::Sketchy(yn) => {
                inner.set_sketchy(*yn);
            },
            QueueEntry::Size(size) => {
                self.regime.set_size(measure,*size);
                self.size = Some(*size);
            },
            QueueEntry::Report => {
                self.intention_lockout = None;
                self.target_reporter.force_report();
            }
        }
    }

    fn detect_endstops(&self, measure: &Measure) -> Vec<Endstop> {
        let mut out = vec![];
        let mut zoom_out = 0;
        if (measure.x_bp - measure.bp_per_screen/2.) < 0.5 {
            out.push(Endstop::Left);
            zoom_out += 1;
        } else {
        }
        if let Some(size) = self.size {
            if (measure.x_bp + measure.bp_per_screen/2.) > size - 0.5 {
                out.push(Endstop::Right);
                zoom_out += 1;
            }
        }
        if zoom_out == 2 {
            out.push(Endstop::MaxZoomOut);
        }
        if measure.bp_per_screen < self.min_bp_per_screen + 0.5 {
            out.push(Endstop::MaxZoomIn);
        }
        out.sort();
        out
    }

    fn report_targets(&mut self, measure: &Measure, report: &mut Report) {
        report.set_endstops(&self.detect_endstops(measure));
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
        let lock = self.target_reporter.lock_updates();
        loop {
            if self.exit_due_to_waiting() { break; }
            self.animation_current = self.animation_queue.pop_front();
            if self.animation_current.is_none() { break; }
            /* do it */
            let current = self.animation_current.take();
            let measure = if let Some(measure) = Measure::new(inner)? { measure } else { break; };
            if let Some(entry) = &current {
                self.run_one_queue_entry(&measure,entry,inner);
            }
            self.report_targets(&measure,report);
            self.animation_current = current;
        }
        drop(lock);
        Ok(())
    }
}
