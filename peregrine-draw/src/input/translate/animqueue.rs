use std::collections::VecDeque;

use crate::{Message, PeregrineAPI, PeregrineInnerAPI, stage::axis::ReadStageAxis};
use super::axisphysics::{AxisPhysics, AxisPhysicsConfig};
use super::measure::Measure;

pub(super) fn bp_to_zpx(bp: f64) -> f64 { bp.log2() * 100. }
pub(super) fn zpx_to_bp(zpx: f64) -> f64 { 2_f64.powf(zpx/100.) }

pub(super) enum QueueEntry {
    MoveW(f64,f64),
    MoveX(f64),
    MoveZ(f64,Option<f64>),
    JumpX(f64,Option<Box<dyn Fn(f64)>>),
    JumpZ(f64,Option<f64>,Option<Box<dyn Fn(f64)>>),
    BrakeX,
    BrakeZ
}

pub(super) struct PhysicsRunner {
    w_left: AxisPhysics,
    w_right: AxisPhysics,
    w_scale: f64,
    x: AxisPhysics,
    z: AxisPhysics,
    animation_queue: VecDeque<QueueEntry>,
    animation_current: Option<QueueEntry>,
    zoom_centre: Option<f64>
}


impl PhysicsRunner {
    pub(super) fn new() -> PhysicsRunner {
        let w_config = AxisPhysicsConfig {
            lethargy: 300., // 2500 for keys & animate, 300 for mouse, 50000 for goto
            boing: 1.,
            vel_min: 0.0005,
            force_min: 0.00001,
            brake_mul: 0.2
        };
        let x_config = AxisPhysicsConfig {
            lethargy: 300.,
            boing: 1.,
            vel_min: 0.0005,
            force_min: 0.00001,
            brake_mul: 0.2
        };
        let z_config = AxisPhysicsConfig {
            lethargy: 300.,
            boing: 1.,
            vel_min: 0.0005,
            force_min: 0.00001,
            brake_mul: 0.2
        };
        PhysicsRunner {
            w_left: AxisPhysics::new(w_config.clone()),
            w_right: AxisPhysics::new(w_config),
            w_scale: 1.,
            x: AxisPhysics::new(x_config),
            z: AxisPhysics::new(z_config),
            animation_queue: VecDeque::new(),
            animation_current: None,
            zoom_centre: None
        }
    }

    pub(super) fn queue_clear(&mut self) {
        self.animation_queue.clear();
    }

    pub(super) fn queue_add(&mut self, entry: QueueEntry) {
        self.animation_queue.push_back(entry);
    }

    pub(super) fn update_needed(&mut self)  -> bool{
        self.w_left.is_active() || self.w_right.is_active() || self.x.is_active() || self.z.is_active() || 
        self.animation_queue.len() !=0 || self.animation_current.is_some()
    }

    fn jump_w(&mut self, inner: &PeregrineInnerAPI, new_x: f64, new_bp_per_screen: f64) -> Result<(),Message> {
        let measure = if let Some(measure) = Measure::new(inner)? { measure } else { return Ok(()); };
        self.x.halt();
        self.z.halt();
        let new_left_bp = new_x - (new_bp_per_screen/2.);
        let new_right_bp = new_x + (new_bp_per_screen/2.);
        self.w_scale = measure.bp_per_screen / measure.px_per_screen; // bp_per_px
        self.w_left.move_to(new_left_bp/self.w_scale);
        self.w_right.move_to(new_right_bp/self.w_scale);
        Ok(())
    }

    fn jump_x(&mut self, inner: &PeregrineInnerAPI, amount_px: f64) -> Result<Option<f64>,Message> {
        let measure = if let Some(measure) = Measure::new(inner)? { measure } else { return Ok(None); };
        let current_px = measure.x_bp / measure.bp_per_screen * measure.px_per_screen;
        if !self.x.is_active() {
            self.x.move_to(current_px);
        }
        self.x.move_more(amount_px);
        self.update_needed();
        Ok(self.x.get_target())
    }

    fn jump_z(&mut self, inner: &PeregrineInnerAPI, amount_px: f64, centre: Option<f64>) -> Result<Option<f64>,Message> {
        let measure = if let Some(measure) = Measure::new(inner)? { measure } else { return Ok(None); };
        let z_current_px = bp_to_zpx(measure.bp_per_screen);
        if !self.z.is_active() {
            self.zoom_centre = centre.clone();
            self.z.move_to(z_current_px);
        }
        self.z.move_more(amount_px);
        self.update_needed();
        Ok(self.z.get_target())
    }

    fn apply_spring_x(&mut self, inner: &mut PeregrineInnerAPI, total_dt: f64) -> Result<(),Message> {
        let measure = if let Some(measure) = Measure::new(inner)? { measure } else { return Ok(()); };
        let px_per_bp = measure.px_per_screen / measure.bp_per_screen;
        if let Some(new_pos_px) = self.x.apply_spring(measure.x_bp*px_per_bp,total_dt) {
            inner.set_x(new_pos_px / px_per_bp);
        }
        Ok(())
    }

    fn apply_spring_w(&mut self, inner: &mut PeregrineInnerAPI, total_dt: f64) -> Result<(),Message> {
        let measure = if let Some(measure) = Measure::new(inner)? { measure } else { return Ok(()); };
        if !self.w_left.is_active() && !self.w_right.is_active() { return Ok(()); }
        /* where are we right now? */
        let old_left_bp = measure.x_bp - measure.bp_per_screen/2.;
        let old_right_bp = measure.x_bp + measure.bp_per_screen/2.;
        let old_left_px = old_left_bp / self.w_scale;
        let old_right_px = old_right_bp / self.w_scale;
        /* how much should we move */
        let new_left_px = self.w_left.apply_spring(old_left_px,total_dt);
        let new_right_px = self.w_right.apply_spring(old_right_px,total_dt);
        let new_pos = match (new_left_px,new_right_px) {
            (Some(left),Some(right)) => Some((left,right)),
            (Some(left),None) => Some((left,old_right_px)),
            (None,Some(right)) => Some((old_left_px,right)),
            (None,None) => None
        };
        if let Some((new_left_px,new_right_px)) = new_pos {
            let new_left_bp = new_left_px * self.w_scale;
            let new_right_bp = new_right_px * self.w_scale;
            /* compute new position */
            inner.set_x((new_left_bp+new_right_bp)/2.);
            inner.set_bp_per_screen(new_right_bp-new_left_bp);
        }
        Ok(())
    }

    fn apply_spring_z(&mut self, inner: &mut PeregrineInnerAPI, total_dt: f64) -> Result<(),Message> {
        let measure = if let Some(measure) = Measure::new(inner)? { measure } else { return Ok(()); };
        if !self.z.is_active() { return Ok(()); }
        let z_current_px = bp_to_zpx(measure.bp_per_screen);
        if let Some(new_pos_px) = self.z.apply_spring(z_current_px,total_dt) {
            let new_bp_per_screen = zpx_to_bp(new_pos_px);
            if let Some(stationary) = self.zoom_centre {
                let x_screen = stationary/measure.px_per_screen;
                let new_bp_from_middle = (x_screen-0.5)*new_bp_per_screen;
                let x_bp = measure.x_bp + (x_screen - 0.5) * measure.bp_per_screen;
                let new_middle = x_bp - new_bp_from_middle;
                inner.set_x(new_middle);
            }
            inner.set_bp_per_screen(new_bp_per_screen);
        }
        Ok(())
    }

    pub(super) fn apply_spring(&mut self, inner: &mut PeregrineInnerAPI, total_dt: f64) -> Result<(),Message> {
        self.apply_spring_w(inner,total_dt)?;
        self.apply_spring_x(inner,total_dt)?;
        self.apply_spring_z(inner,total_dt)?;
        Ok(())
    }

    fn halt_w(&mut self) {
        self.w_right.halt();
        self.w_left.halt();
    }

    pub(super) fn drain_animation_queue(&mut self, inner: &PeregrineInnerAPI) -> Result<(),Message> {
        loop {
            /* still ongoing? */
            if let Some(current) = &mut self.animation_current {
                let axes_active = self.x.is_active() || self.z.is_active();
                let blocked = match current {
                    QueueEntry::MoveX(_) => axes_active,
                    QueueEntry::MoveZ(_,_) => axes_active,
                    QueueEntry::MoveW(_,_) => axes_active,
                    _ => false
                };
                if blocked { break; }
            }
            /* nothing to do? */
            self.animation_current = self.animation_queue.pop_front();
            /* move something from the queue */
            if self.animation_current.is_none() { break; }
            /* w doesn't interact well with ongoing ops but does with itself */
            match &self.animation_current {
                Some(QueueEntry::MoveW(_,_)) => {},
                _ => { self.halt_w(); }
            }
            /* do it */
            let current = self.animation_current.take();
            match &current {
                Some(QueueEntry::MoveW(centre,scale)) => {
                    self.jump_w(inner,*centre, *scale)?;
                },
                Some(QueueEntry::MoveX(amt)) => { 
                    self.x.move_to(*amt);
                },
                Some(QueueEntry::MoveZ(amt,centre)) => {
                    self.zoom_centre = centre.clone();
                    self.z.move_to(*amt);
                },
                Some(QueueEntry::JumpX(amt,cb)) => {
                    let cb = cb.clone();
                    let target = self.jump_x(inner,*amt)?;
                    if let Some(target) = target {
                        if let Some(cb) = cb {
                            cb(target);
                        }
                    }
                },
                Some(QueueEntry::JumpZ(amt,pos,cb)) => { 
                    let cb = cb.clone();
                    let target = self.jump_z(inner,*amt,pos.clone())?; 
                    if let Some(target) = target {
                        if let Some(cb) = cb {
                            cb(target);
                        }
                    }
                },
                Some(QueueEntry::BrakeX) => {
                    self.x.brake() 
                },
                Some(QueueEntry::BrakeZ) => { 
                    self.z.brake() 
                },
                _ => {}
            }
            self.animation_current = current;
        }
        Ok(())
    }
}
