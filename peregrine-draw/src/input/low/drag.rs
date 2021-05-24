use std::sync::{ Arc, Mutex };
use crate::util::{ Message };
use super::{ lowlevel::LowLevelState};
use super::lowlevel::Modifiers;
use js_sys::Date;
use super::mouseinput::{  MouseConfig, MouseAction };

pub struct DragStateData {
    lowlevel: LowLevelState,
    modifiers: Modifiers,
    start_pos: (f64,f64),
    prev_pos: (f64,f64),
    start_time: f64,
    dragged: bool,
    hold: bool,
    alive: bool
}

impl DragStateData {
    fn new(lowlevel: &LowLevelState, current: &(f64,f64)) -> DragStateData {
        DragStateData {
            lowlevel: lowlevel.clone(),
            modifiers: lowlevel.modifiers(),
            start_pos: *current,
            prev_pos: *current,
            start_time: Date::now(),
            dragged: false,
            hold: false,
            alive: true
        }
    }

    fn hold_timer_expired(&mut self) {
        if self.alive && !self.dragged {
            self.hold = true;
            for (kind,args) in MouseAction::SwitchToHold(self.modifiers.clone(),self.start_pos).map(&self.lowlevel) {
                self.lowlevel.send(kind,true,&args);
            }
        }
    }

    fn emit(&mut self, action: &MouseAction, start: bool) {
        for (kind,args) in action.map(&self.lowlevel) {
            self.lowlevel.send(kind,true,&args);
        }    
    }

    fn send_drag(&mut self, delta: (f64,f64), start: bool) {
          // XXX yuck, clones on critical path
        if self.hold {
            self.emit(&MouseAction::RunningHold(self.modifiers.clone(),delta),start);
        } else {
            self.emit(&MouseAction::RunningDrag(self.modifiers.clone(),delta),start);
        }
    }

    fn check_dragged(&mut self, config: &MouseConfig, current: &(f64,f64)) {
        if !self.dragged {
            let total_distance = (current.0-self.start_pos.0,current.1-self.start_pos.1);
            if total_distance.0.abs() + total_distance.1.abs() > config.click_radius {
                self.dragged = true;
            }
        }
    }

    fn drag_continue(&mut self, config: &MouseConfig, current: &(f64,f64)) {
        self.check_dragged(config,current);
        let delta = (current.0-self.prev_pos.0,current.1-self.prev_pos.1);
        self.send_drag(delta,true);
        self.prev_pos = *current;
    }

    fn drag_finished(&mut self, config: &MouseConfig, current: &(f64,f64)) {
        self.check_dragged(config,current);
        let delta = (current.0-self.prev_pos.0,current.1-self.prev_pos.1);
        self.send_drag(delta,true);
        self.alive = false;
        if self.dragged {
            self.send_drag((0.,0.),false);
            if self.hold {
                self.emit(&MouseAction::HoldDrag(self.modifiers.clone(),(current.0-self.start_pos.0,current.1-self.start_pos.1)),true);
            } else {
                self.emit(&MouseAction::Drag(self.modifiers.clone(),(current.0-self.start_pos.0,current.1-self.start_pos.1)),true);
            }
        } else {
            self.emit(&MouseAction::Click(self.modifiers.clone(),self.start_pos.clone()),true);
        }  
    }
}

pub struct DragState(Arc<Mutex<DragStateData>>);

impl DragState {
    pub(super) fn new(config: &MouseConfig, lowlevel: &LowLevelState, current: &(f64,f64)) -> DragState {
        let inner = Arc::new(Mutex::new(DragStateData::new(lowlevel,current)));
        let inner2 = inner.clone();
        let hold_time = config.hold_delay;
        lowlevel.timer(hold_time, move || {
            inner2.lock().unwrap().hold_timer_expired();
        });
        DragState(inner)
    }

    pub(super) fn drag_continue(&mut self, config: &MouseConfig, current: &(f64,f64)) {
        self.0.lock().unwrap().drag_continue(config,current);
    }

    pub(super) fn drag_finished(&mut self, config: &MouseConfig, current: &(f64,f64)) {
        self.0.lock().unwrap().drag_finished(config,current);
    }
}
