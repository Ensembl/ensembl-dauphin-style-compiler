use std::rc::Rc;
use std::cell::RefCell;
use peregrine_toolkit::sync::needed::Needed;

use crate::util::message::Message;

#[derive(Clone)]
struct BootLock(Boot,bool);

impl BootLock {
    fn unlock(&mut self) {
        if self.1 {
            self.1 = false;
            self.0.unlock();
        }
    }
}

#[derive(Clone)]
struct Boot(Rc<RefCell<usize>>);

impl Boot {
    fn new() -> Boot {
        Boot(Rc::new(RefCell::new(0)))
    }

    fn lock(&self) -> BootLock {
        *self.0.borrow_mut() += 1;
        BootLock(self.clone(),true)
    }

    fn unlock(&self) {
        *self.0.borrow_mut() -= 1;
    }

    fn booted(&self) -> bool {
        *self.0.borrow() == 0
    }
}

fn stage_ok<T: Clone>(x: &Option<T>) -> Result<T,Message> {
    x.as_ref().cloned().ok_or_else(|| Message::CodeInvariantFailed("accseeor used on non-ready stage".to_string()))
}

pub trait ReadStageAxis {
    fn position(&self) -> Result<f64,Message>;
    fn bp_per_screen(&self) -> Result<f64,Message>;
    fn bp_per_screen2(&self) -> Result<f64,Message>;
    fn container_size(&self) -> Result<f64,Message>;
    fn scale_shift(&self) -> Result<(f32,f32),Message>;
    fn squeeze(&self) -> Result<(f32,f32),Message>;
    fn drawable_size(&self) -> Result<f64,Message>;   
    fn copy(&self) -> StageAxis;
    fn version(&self) -> u64;
    fn ready(&self) -> bool;
    fn left_right(&self) -> Result<(f64,f64),Message>;
    fn convert_canvas_prop_delta_to_bp(&self, prop: f64) -> f64;
    fn convert_delta_bp_to_canvas_prop(&self, bp: f64) -> f64;
    fn convert_delta_bp_to_px(&self, bp: f64) -> f64;
    fn convert_px_delta_to_bp(&self, px: i64) -> f64;
    fn convert_bp_delta_to_px(&self, bp: f64) -> i64;
    fn convert_px_pos_to_bp(&self, px: i64) -> Result<f64,Message>;
    fn convert_bp_to_pos_px(&self, bp: f64) -> Result<i64,Message>;
}

pub struct StageAxis {
    position: Option<f64>,
    bp_per_screen: Option<f64>,
    size: Option<f64>,
    draw_size: Option<f64>,
    scale_shift: Option<(f32,f32)>,
    squeeze: (f32,f32),
    redraw_needed: Needed,
    boot: Boot,
    boot_lock: BootLock,
    version: u64
}

impl StageAxis {
    pub(super) fn new(redraw_needed: &Needed) -> StageAxis {
        let boot = Boot::new();
        let boot_lock = boot.lock();
        StageAxis {
            position: None,
            bp_per_screen: None,
            size: None,
            draw_size: None,
            scale_shift: None,
            redraw_needed: redraw_needed.clone(),
            boot: boot.clone(),
            squeeze: (0.,0.),
            boot_lock,
            version: 0
        }
    }

    fn recompute_scale_shift(&mut self) {
        if self.size.is_none() || self.draw_size.is_none() {
            self.draw_size = None;
            return;
        }
        /* we need -1 to stay stationary. our scaling sets it to -scale so we need to add scale-1 */
        let scale = self.draw_size.unwrap() as f32 / self.size.unwrap() as f32;
        self.scale_shift = Some((
            scale,
            scale-1.
        ));
    }

    fn data_ready(&self) -> bool {
        self.position.is_some() && self.bp_per_screen.is_some()
    }

    fn changed(&mut self) {
        if !self.boot.booted() {
            if self.data_ready() {
                self.boot_lock.unlock();
            }
        }
        if self.boot.booted() {
            self.redraw_needed.set();
        }
        self.version += 1;
    }

    pub fn set_squeeze(&mut self, squeeze: (f32,f32)) { self.squeeze = squeeze; }

    pub fn set_position(&mut self, x: f64) { self.position = Some(x); self.changed(); }
    pub fn set_size(&mut self, x: f64) { self.size = Some(x); self.recompute_scale_shift(); self.changed(); }
    pub fn set_drawable_size(&mut self, x: f64) { self.draw_size = Some(x); self.recompute_scale_shift(); self.changed(); }
    pub fn set_bp_per_screen(&mut self, z: f64) { self.bp_per_screen = Some(z); self.changed(); }
}

impl ReadStageAxis for StageAxis {
    fn position(&self) -> Result<f64,Message> { stage_ok(&self.position) }
    fn bp_per_screen(&self) -> Result<f64,Message> { stage_ok(&self.bp_per_screen) }
    fn bp_per_screen2(&self) -> Result<f64,Message> { stage_ok(&self.bp_per_screen) }
    fn container_size(&self) -> Result<f64,Message> { stage_ok(&self.size) }
    fn drawable_size(&self) -> Result<f64,Message> { stage_ok(&self.draw_size) }
    fn scale_shift(&self) -> Result<(f32,f32),Message> { stage_ok(&self.scale_shift) }
    fn squeeze(&self) -> Result<(f32,f32),Message> { stage_ok(&Some(self.squeeze)) }
    fn left_right(&self) -> Result<(f64,f64),Message> {
        let pos = self.position()?;
        let bp_per_screen = self.bp_per_screen2()?;
        Ok((pos-bp_per_screen/2.-1.,pos+bp_per_screen/2.+1.))
    }

    fn convert_canvas_prop_delta_to_bp(&self, prop: f64) -> f64 {
        if let (Some(draw_size),Some(bp_per_screen)) = (self.draw_size,self.bp_per_screen) {
            let invisible_prop = (self.squeeze.0+self.squeeze.1) as f64/draw_size;
            prop * bp_per_screen / (1.0-invisible_prop)
        } else {
            0.
        }
    }

    fn convert_delta_bp_to_canvas_prop(&self, bp: f64) -> f64 {
        if let (Some(draw_size),Some(bp_per_screen)) = (self.draw_size,self.bp_per_screen) {
            let invisible_prop = (self.squeeze.0+self.squeeze.1) as f64/draw_size;
            bp / bp_per_screen * (1.0-invisible_prop)
        } else {
            0.
        }
    }

    fn convert_delta_bp_to_px(&self, bp: f64) -> f64 {
        if let Some(draw_size) = self.draw_size {
            self.convert_delta_bp_to_canvas_prop(bp) * draw_size
        } else {
            0.
        }
    }

    fn convert_px_delta_to_bp(&self, px: i64) -> f64 {
        if let Some(draw_size) = self.draw_size {
            self.convert_canvas_prop_delta_to_bp(px as f64 / draw_size)
        } else {
            0.
        }
    }

    fn convert_bp_delta_to_px(&self, bp: f64) -> i64 {
        if let Some(draw_size) = self.draw_size {
            (self.convert_delta_bp_to_canvas_prop(bp) * draw_size) as i64
        } else {
            0
        }
    }

    fn convert_px_pos_to_bp(&self, px: i64) -> Result<f64,Message> {
        let px = px as f64;
        let position = stage_ok(&self.position)?;
        let draw_size = stage_ok(&self.draw_size)?;
        let position_x_scr = px / draw_size;
        let user_centre = (1.0+(self.squeeze.0-self.squeeze.1) as f64/draw_size)/2.0;
        let position_x_bp = self.convert_canvas_prop_delta_to_bp(position_x_scr - user_centre) + position;
        Ok(position_x_bp)
    }

    fn convert_bp_to_pos_px(&self, bp: f64) -> Result<i64,Message> {
        let position = stage_ok(&self.position)?;
        let draw_size = stage_ok(&self.draw_size)?;
        let bp_right_of_user_centre = bp - position;
        let prop_right_of_user_centre = self.convert_delta_bp_to_canvas_prop(bp_right_of_user_centre);
        let user_centre = (1.0+(self.squeeze.0-self.squeeze.1) as f64/draw_size)/2.0;
        let prop = user_centre + prop_right_of_user_centre;
        Ok((prop*draw_size) as i64)
    }

    // secret clone only accessible via read-only subsets
    fn copy(&self) -> StageAxis {
        StageAxis {
            position: self.position.clone(),
            bp_per_screen: self.bp_per_screen.clone(),
            size: self.size.clone(),
            draw_size: self.draw_size.clone(),
            scale_shift: self.scale_shift.clone(),
            redraw_needed: self.redraw_needed.clone(),
            squeeze: self.squeeze.clone(),
            version: self.version,
            boot: self.boot.clone(),
            boot_lock: self.boot_lock.clone()
        }
    }    

    fn ready(&self) -> bool {
        self.position.is_some() && self.bp_per_screen.is_some() && self.size.is_some() && self.draw_size.is_some()
    }

    fn version(&self) -> u64 { self.version }
}
