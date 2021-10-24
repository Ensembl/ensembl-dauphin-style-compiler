use std::collections::VecDeque;
use crate::run::{PgConfigKey, PgPeregrineConfig};
use crate::run::report::Report;
use crate::util::message::Endstop;
use crate::{Message, PeregrineInnerAPI };
use super::measure::Measure;
use crate::input::regimes::regime::Regime;

#[derive(Clone)]
pub(super) enum Cadence {
    #[allow(unused)]
    UserInput,
    Instructed,
    SelfPropelled
}

pub(super) enum QueueEntry {
    MoveW(f64,f64),
    ShiftTo(f64,Cadence),
    ShiftByZoomTo(f64,Cadence),
    ZoomTo(f64,Cadence),
    ShiftMore(f64),
    ZoomMore(f64,Option<f64>),
    BrakeX,
    BrakeZ,
    Wait,
    Size(f64)
}

pub(super) struct PhysicsRunner {
    regime: Regime,
    animation_queue: VecDeque<QueueEntry>,
    animation_current: Option<QueueEntry>,
    size: Option<f64>,
    max_zoom_in_bp: f64
}

impl PhysicsRunner {
    pub(super) fn new(config: &PgPeregrineConfig) -> Result<PhysicsRunner,Message> {
        Ok(PhysicsRunner {
            regime: Regime::new(config)?,
            animation_queue: VecDeque::new(),
            animation_current: None,
            size: None,
            max_zoom_in_bp: config.get_f64(&PgConfigKey::MinBpPerScreen)?
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

    pub(crate) fn regime_tick(&mut self, inner: &mut PeregrineInnerAPI, total_dt: f64) -> Result<(),Message> {
        self.regime.tick(inner,total_dt)
    }

    fn run_one_queue_entry(&mut self, measure: &Measure, entry: &QueueEntry) {
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
            QueueEntry::ShiftByZoomTo(amt,_cadence) => {
                self.regime.regime_zoomx(measure).set(measure,*amt);
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
                self.size = Some(*size);
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
        if measure.bp_per_screen < self.max_zoom_in_bp + 0.5 {
            out.push(Endstop::MaxZoomIn);
        }
        out.sort();
        out
    }

    fn report_targets(&mut self, measure: &Measure, report: &mut Report) {
        let (target_x,target_bp) = self.regime.report_target(&measure);
        if let Some(target_x) = target_x {
            report.set_target_x_bp(target_x);
        }
        if let Some(target_bp) = target_bp {
            report.set_target_bp_per_screen(target_bp);
        }
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
        loop {
            if self.exit_due_to_waiting() { break; }
            self.animation_current = self.animation_queue.pop_front();
            if self.animation_current.is_none() { break; }
            /* do it */
            let current = self.animation_current.take();
            let measure = if let Some(measure) = Measure::new(inner)? { measure } else { return Ok(()); };
            if let Some(entry) = &current {
                self.run_one_queue_entry(&measure,entry);
            }
            self.report_targets(&measure,report);
            self.animation_current = current;
        }
        Ok(())
    }
}
