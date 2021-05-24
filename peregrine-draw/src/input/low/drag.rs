use commander::cdr_timer;
use std::sync::{ Arc, Mutex };
use crate::{PeregrineDom, PgCommanderWeb, run::{PgPeregrineConfig, PgConfigKey}};
use peregrine_data::PgCommander;
use web_sys::{ MouseEvent, HtmlElement, Event, WheelEvent };
use crate::input::{ InputEvent, InputEventKind, Distributor };
use crate::util::{ Message };
use super::{event::{ EventSystem }, lowlevel::LowLevelState};
use super::mapping::InputMap;
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

    fn hold_timer_expired(&mut self) -> Option<MouseAction> {
        if self.alive && !self.dragged {
            self.hold = true;
            return Some(MouseAction::Hold(self.lowlevel.modifiers(),self.start_pos));
        } else {
            return None;
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

    fn drag_continue(&mut self, emit: &mut Vec<MouseAction>, config: &MouseConfig, current: &(f64,f64)) {
        self.check_dragged(config,current);
        let delta = (current.0-self.prev_pos.0,current.1-self.prev_pos.1);
        emit.push(MouseAction::RunningDrag(self.modifiers.clone(),delta));  // XXX yuck, clone on critical path
        self.prev_pos = *current;
    }

    fn drag_finished(&mut self, emit: &mut Vec<MouseAction>, config: &MouseConfig, current: &(f64,f64)) {
        self.check_dragged(config,current);
        let delta = (current.0-self.prev_pos.0,current.1-self.prev_pos.1);
        emit.push(MouseAction::RunningDrag(self.modifiers.clone(),delta));
        self.alive = false;
        if self.dragged {
            emit.push(MouseAction::Drag(self.modifiers.clone(),(current.0-self.start_pos.0,current.1-self.start_pos.1)));
        } else {
            emit.push(MouseAction::Click(self.modifiers.clone(),self.start_pos.clone()));
        }  
    }
}

pub struct DragState(Arc<Mutex<DragStateData>>);

impl DragState {
    pub(super) fn new(config: &MouseConfig, lowlevel: &LowLevelState, current: &(f64,f64)) -> DragState {
        let inner = Arc::new(Mutex::new(DragStateData::new(lowlevel,current)));
        let inner2 = inner.clone();
        let hold_time = config.hold_delay;
        let lowlevel2 = lowlevel.clone();
        lowlevel.timer(hold_time, move || {
            if let Some(action) = inner2.lock().unwrap().hold_timer_expired() {
                // TODO send it
            }
        });
        DragState(inner)
    }

    pub(super) fn drag_continue(&mut self, emit: &mut Vec<MouseAction>, config: &MouseConfig, current: &(f64,f64)) {
        self.0.lock().unwrap().drag_continue(emit,config,current);
    }

    pub(super) fn drag_finished(&mut self, emit: &mut Vec<MouseAction>, config: &MouseConfig, current: &(f64,f64)) {
        self.0.lock().unwrap().drag_finished(emit,config,current);
    }
}
