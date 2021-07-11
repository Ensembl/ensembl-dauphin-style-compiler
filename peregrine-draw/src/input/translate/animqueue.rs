use std::collections::VecDeque;
use crate::{Message, PeregrineAPI};
use super::axisphysics::{AxisPhysics, AxisPhysicsConfig};

pub(super) fn bp_to_zpx(bp: f64) -> f64 { bp.log2() * 100. }
pub(super) fn zpx_to_bp(zpx: f64) -> f64 { 2_f64.powf(zpx/100.) }

pub(super) enum QueueEntry {
    MoveW(f64,f64),
    MoveX(f64),
    MoveZ(f64,ZoomCentre),
    JumpX(f64),
    JumpZ(f64,ZoomCentre),
    BrakeX,
    BrakeZ
}

#[derive(Clone)]
pub(super) enum ZoomCentre {
    None,
    StationaryPoint(f64),
    CentreOfScreen(f64)
}

pub(super) struct PhysicsRunner {
    w_left: AxisPhysics,
    w_right: AxisPhysics,
    w_scale: f64,
    x: AxisPhysics,
    z: AxisPhysics,
    animation_queue: VecDeque<QueueEntry>,
    animation_current: Option<QueueEntry>,
    zoom_centre: ZoomCentre
}

impl PhysicsRunner {
    pub(super) fn new() -> PhysicsRunner {
        let w_config = AxisPhysicsConfig {
            lethargy: 300., // 2500 for keys & animate, 300 for mouse
            boing: 1.,
            vel_min: 0.0005,
            force_min: 0.00001,
            brake_mul: 0.2
        };
        let x_config = AxisPhysicsConfig {
            lethargy: 300., // 2500 for keys
            boing: 1.,
            vel_min: 0.0005,
            force_min: 0.00001,
            brake_mul: 0.2
        };
        let z_config = AxisPhysicsConfig {
            lethargy: 100.,
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
            zoom_centre: ZoomCentre::None
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

    fn jump_w(&mut self, api: &PeregrineAPI, new_x: f64, new_bp_per_screen: f64) -> Result<(),Message> {
        self.x.halt();
        self.z.halt();
        if let (Some(old_x),Some(old_bp_per_screen),Some(size_px)) = (api.x()?,api.bp_per_screen()?,api.size()) {
            let new_left_bp = new_x - (new_bp_per_screen/2.);
            let new_right_bp = new_x + (new_bp_per_screen/2.);
            self.w_scale = old_bp_per_screen / (size_px.0 as f64); // bp_per_px
            self.w_left.move_to(new_left_bp/self.w_scale);
            self.w_right.move_to(new_right_bp/self.w_scale);
        }
        Ok(())
    }

    fn jump_x(&mut self, api: &PeregrineAPI, amount_px: f64) -> Result<(),Message> {
        if let (Some(current_bp),Some(bp_per_screen),Some(size_px)) = (api.x()?,api.bp_per_screen()?,api.size()) {
            let current_px = current_bp / bp_per_screen * (size_px.0 as f64);
            if !self.x.is_active() {
                self.x.move_to(current_px);
            }
            self.x.move_more(amount_px);
        }
        self.update_needed();
        Ok(())
    }

    fn jump_z(&mut self, api: &PeregrineAPI, amount_px: f64, centre: &ZoomCentre) -> Result<(),Message> {
        if let Some(bp_per_screen) = api.bp_per_screen()? {
            let z_current_px = bp_to_zpx(bp_per_screen);
            if !self.z.is_active() {
                self.zoom_centre = centre.clone();
                self.z.move_to(z_current_px);
            }
            self.z.move_more(amount_px);
        }
        self.update_needed();
        Ok(())
    }

    fn apply_spring_x(&mut self, api: &PeregrineAPI, total_dt: f64) -> Result<(),Message> {
        if !self.x.is_active() { return Ok(()); }
        let x_current_bp = api.x()?;
        if let (Some(x_current_bp),Some(screen_size),Some(bp_per_screen)) = 
                    (x_current_bp,api.size(),api.bp_per_screen()?) {
            let px_per_screen = screen_size.0 as f64;
            let px_per_bp = px_per_screen / bp_per_screen;
            let new_pos_px = self.x.apply_spring(x_current_bp*px_per_bp,total_dt);
            api.set_x(new_pos_px / px_per_bp);
        }
        Ok(())
    }

    fn apply_spring_w(&mut self, api: &PeregrineAPI, total_dt: f64) -> Result<(),Message> {
        if !self.w_left.is_active() && !self.w_right.is_active() { return Ok(()); }
        let current_x = api.x()?;
        if let (Some(current_x),Some(screen_size),Some(current_bp_per_screen)) = 
                    (current_x,api.size(),api.bp_per_screen()?) {
            /* where are we right now? */
            let old_left_bp = current_x - current_bp_per_screen/2.;
            let old_right_bp = current_x + current_bp_per_screen/2.;
            let old_left_px = old_left_bp / self.w_scale;
            let old_right_px = old_right_bp / self.w_scale;
            /* how much should we move */
            let new_left_px = self.w_left.apply_spring(old_left_px,total_dt);
            let new_right_px = self.w_right.apply_spring(old_right_px,total_dt);
            let new_left_bp = new_left_px * self.w_scale;
            let new_right_bp = new_right_px * self.w_scale;
            /* compute new position */
            api.set_x((new_left_bp+new_right_bp)/2.);
            api.set_bp_per_screen(new_right_bp-new_left_bp);
        }
        Ok(())
    }

    fn apply_spring_z(&mut self, api: &PeregrineAPI, total_dt: f64) -> Result<(),Message> {
        if !self.z.is_active() { return Ok(()); }
        let px_per_screen = api.size().map(|x| x.0 as f64);
        let z_current_bp = api.bp_per_screen()?;
        let x = api.x()?;
        if let (Some(x),Some(z_current_bp),Some(screen_size),Some(bp_per_screen)) = 
                    (x,z_current_bp,px_per_screen,api.bp_per_screen()?) {                        
            let z_current_px = bp_to_zpx(z_current_bp);
            let new_pos_px = self.z.apply_spring(z_current_px,total_dt);
            let new_bp_per_screen = zpx_to_bp(new_pos_px);
            match self.zoom_centre {
                ZoomCentre::StationaryPoint(stationary) => {
                    let x_screen = stationary/screen_size;
                    let new_bp_from_middle = (x_screen-0.5)*bp_per_screen;
                    let x_bp = x + (x_screen - 0.5) * bp_per_screen;
                    let new_middle = x_bp - new_bp_from_middle;
                    api.set_x(new_middle);    
                },
                ZoomCentre::CentreOfScreen(centre) => {
                    api.set_x(centre);
                },
                ZoomCentre::None => {}
            }
            api.set_bp_per_screen(new_bp_per_screen);
        }
        Ok(())
    }

    pub(super) fn apply_spring(&mut self, api: &PeregrineAPI, total_dt: f64) -> Result<(),Message> {
        self.apply_spring_w(api,total_dt)?;
        self.apply_spring_x(api,total_dt)?;
        self.apply_spring_z(api,total_dt)?;
        Ok(())
    }

    fn halt_w(&mut self) {
        self.w_right.halt();
        self.w_left.halt();
    }

    pub(super) fn drain_animation_queue(&mut self, api: &PeregrineAPI) -> Result<(),Message> {
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
            match &self.animation_current {
                Some(QueueEntry::MoveW(centre,scale)) => { self.jump_w(api,*centre, *scale)?; },
                Some(QueueEntry::MoveX(amt)) => { self.x.move_to(*amt); },
                Some(QueueEntry::MoveZ(amt,centre)) => { self.zoom_centre = centre.clone(); self.z.move_to(*amt); },
                Some(QueueEntry::JumpX(amt)) => { self.jump_x(api,*amt)?; },
                Some(QueueEntry::JumpZ(amt,pos)) => { self.jump_z(api,*amt,&pos.clone())?; },
                Some(QueueEntry::BrakeX) => { self.x.brake() },
                Some(QueueEntry::BrakeZ) => { self.z.brake() },
                _ => {}
            }    
        }
        Ok(())
    }
}
