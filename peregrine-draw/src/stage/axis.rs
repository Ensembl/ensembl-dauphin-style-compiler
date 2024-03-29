use std::mem;
use std::rc::Rc;
use std::cell::RefCell;
use peregrine_toolkit_async::sync::needed::Needed;

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
    fn container_size(&self) -> Result<f64,Message>;
    fn scale_shift(&self) -> Result<(f32,f32),Message>;
    fn squeeze(&self) -> Result<(f32,f32),Message>;
    fn drawable_size(&self) -> Result<f64,Message>;   
    fn copy(&self) -> StageAxis;
    fn version(&self) -> u64;
    fn ready(&self) -> bool;
    fn left_right(&self) -> Result<(f64,f64),Message>;
    fn unit_converter(&self) -> Result<UnitConverter,Message>;
}

pub struct UnitConverter {
    position: f64,
    bp_per_screen: f64,
    draw_size: f64,
    squeeze: (f32,f32)
}

impl UnitConverter {
    pub fn move_to(&self, position: f64) -> UnitConverter {
        UnitConverter {
            position,
            bp_per_screen: self.bp_per_screen,
            draw_size: self.draw_size,
            squeeze: self.squeeze
        }
    }

    pub fn resize_prop(&self, scale: f64) -> UnitConverter {
        UnitConverter {
            position: self.position,
            bp_per_screen: self.bp_per_screen * scale,
            draw_size: self.draw_size,
            squeeze: self.squeeze
        }
    }

    pub fn bp_per_screen(&self) -> f64 { self.bp_per_screen }
    pub fn position(&self) -> f64 { self.position }
    pub fn left_rail(&self) -> f64 { self.squeeze.0 as f64 }

    pub fn canvas_prop_delta_to_bp(&self, prop: f64) -> f64 {
        let invisible_prop = (self.squeeze.0+self.squeeze.1) as f64/self.draw_size;
        prop * self.bp_per_screen / (1.0-invisible_prop)
    }

    pub fn canvas_prop_to_bp_from_centre(&self, prop: f64) -> f64 {
        self.px_pos_to_bp_from_centre(prop * self.draw_size)
    }

    pub fn delta_bp_to_canvas_prop(&self, bp: f64) -> f64 {
        let invisible_prop = (self.squeeze.0+self.squeeze.1) as f64/self.draw_size;
        bp / self.bp_per_screen * (1.0-invisible_prop)
    }

    pub fn delta_bp_to_px(&self, bp: f64) -> f64 {
        self.delta_bp_to_canvas_prop(bp) * self.draw_size
    }

    pub fn px_delta_to_bp(&self, px: f64) -> f64 {
        self.canvas_prop_delta_to_bp(px as f64 / self.draw_size)
    }

    pub fn px_pos_to_bp_from_centre(&self, px: f64) -> f64 {
        let px = px as f64;
        let position_x_scr = px / self.draw_size;
        let user_centre = (1.0+(self.squeeze.0-self.squeeze.1) as f64/self.draw_size)/2.0;
        self.canvas_prop_delta_to_bp(position_x_scr - user_centre)        
    }

    pub fn px_pos_to_bp(&self, px: f64) -> f64 {
        self.px_pos_to_bp_from_centre(px) + self.position
    }

    pub fn px_pos_to_screen_prop(&self, px: f64) -> f64 {
        (px as f64) / self.draw_size
    }

    pub fn bp_to_pos_px(&self, bp: f64) -> Result<f64,Message> {
        let bp_right_of_user_centre = bp - self.position;
        let prop_right_of_user_centre = self.delta_bp_to_canvas_prop(bp_right_of_user_centre);
        let user_centre = (1.0+(self.squeeze.0-self.squeeze.1) as f64/self.draw_size)/2.0;
        let prop = user_centre + prop_right_of_user_centre;
        Ok((prop*self.draw_size) as f64)
    }
}

pub struct StageAxis {
    position: Option<f64>,
    bp_per_screen: Option<f64>,
    update_listeners: Vec<Box<dyn FnMut(&StageAxis)>>,
    size: Option<f64>,
    max_bottom: Option<f64>,
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
            update_listeners: vec![],
            size: None,
            draw_size: None,
            scale_shift: None,
            max_bottom: None,
            redraw_needed: redraw_needed.clone(),
            boot: boot.clone(),
            squeeze: (0.,0.),
            boot_lock,
            version: 0
        }
    }

    pub fn add_listener<F>(&mut self, listener: F) where F: FnMut(&StageAxis) + 'static {
        self.update_listeners.push(Box::new(listener));
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

    fn run_listeners(&mut self) {
        let mut listeners = mem::replace(&mut self.update_listeners,vec![]);
        for listener in &mut listeners {
            (listener)(self);
        }
        self.update_listeners = listeners;
    }

    fn changed(&mut self) {
        self.run_listeners();
        if let Some(position) = self.position.as_mut() {
            if let (Some(max_bottom),Some(size)) = (self.max_bottom,self.size) {
                *position = position.min((max_bottom-size) as f64);
            }    
            *position = position.max(0.);
        }
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
    pub fn set_max_bottom(&mut self, viewport: f64) { self.max_bottom = Some(viewport); self.changed(); }
    pub fn set_position(&mut self, x: f64) { self.position = Some(x); self.changed(); }
    pub fn set_size(&mut self, x: f64) { self.size = Some(x); self.recompute_scale_shift(); self.changed(); }
    pub fn set_drawable_size(&mut self, x: f64) { self.draw_size = Some(x); self.recompute_scale_shift(); self.changed(); }
    pub fn set_bp_per_screen(&mut self, z: f64) { 
        self.bp_per_screen = Some(z); 
        self.changed();
    }
}

impl ReadStageAxis for StageAxis {
    fn position(&self) -> Result<f64,Message> { stage_ok(&self.position) }
    fn bp_per_screen(&self) -> Result<f64,Message> { stage_ok(&self.bp_per_screen) }
    fn container_size(&self) -> Result<f64,Message> { stage_ok(&self.size) }
    fn drawable_size(&self) -> Result<f64,Message> { stage_ok(&self.draw_size) }
    fn scale_shift(&self) -> Result<(f32,f32),Message> { stage_ok(&self.scale_shift) }
    fn squeeze(&self) -> Result<(f32,f32),Message> { stage_ok(&Some(self.squeeze)) }
    fn left_right(&self) -> Result<(f64,f64),Message> {
        let pos = self.position()?;
        let bp_per_screen = self.bp_per_screen()?;
        Ok((pos-bp_per_screen/2.-1.,pos+bp_per_screen/2.+1.))
    }

    fn unit_converter(&self) -> Result<UnitConverter,Message> {
        let draw_size = stage_ok(&self.draw_size)?;
        let position = stage_ok(&self.position)?;
        let bp_per_screen = stage_ok(&self.bp_per_screen)?;
        let squeeze = self.squeeze;
        Ok(UnitConverter {
            draw_size,
            position,
            bp_per_screen,
            squeeze
        })
    }

    // secret clone only accessible via read-only subsets
    fn copy(&self) -> StageAxis {
        StageAxis {
            position: self.position.clone(),
            update_listeners: vec![], // Not accessible in reader anyway and can't be cloned
            bp_per_screen: self.bp_per_screen.clone(),
            size: self.size.clone(),
            draw_size: self.draw_size.clone(),
            scale_shift: self.scale_shift.clone(),
            max_bottom: self.max_bottom.clone(),
            redraw_needed: self.redraw_needed.clone(),
            squeeze: self.squeeze.clone(),
            version: self.version,
            boot: self.boot.clone(),
            boot_lock: self.boot_lock.clone()
        }
    }    

    fn ready(&self) -> bool {
        self.position.is_some() && self.bp_per_screen.is_some() && 
        self.size.is_some() && self.draw_size.is_some()
    }

    fn version(&self) -> u64 { self.version }
}
