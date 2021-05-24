use std::sync::{ Arc, Mutex };
use crate::input::low::lowlevel::LowLevelState;
use crate::input::low::lowlevel::Modifiers;
use super::pointer::{ PointerConfig, PointerAction };
use super::cursor::CursorHandle;
use crate::run::CursorCircumstance;

pub struct DragStateData {
    lowlevel: LowLevelState,
    modifiers: Modifiers,
    start_pos: (f64,f64),
    prev_pos: (f64,f64),
    dragged: bool,
    hold: bool,
    alive: bool,
    #[allow(unused)] // keep as guard
    cursor: Option<CursorHandle>
}

impl DragStateData {
    fn new(lowlevel: &LowLevelState, current: &(f64,f64)) -> DragStateData {
        DragStateData {
            lowlevel: lowlevel.clone(),
            modifiers: lowlevel.modifiers(),
            start_pos: *current,
            prev_pos: *current,
            dragged: false,
            hold: false,
            alive: true,
            cursor: None
        }
    }

    fn click_timer_expired(&mut self) {
        if self.alive && !self.hold {
            self.cursor = Some(self.lowlevel.set_cursor(&CursorCircumstance::Drag));
        }
    }

    fn hold_timer_expired(&mut self) {
        if self.alive && !self.dragged {
            self.hold = true;
            self.cursor = Some(self.lowlevel.set_cursor(&CursorCircumstance::Hold));
            for (kind,args) in PointerAction::SwitchToHold(self.modifiers.clone(),self.start_pos).map(&self.lowlevel) {
                self.lowlevel.send(kind,true,&args);
            }
        }
    }

    fn emit(&mut self, action: &PointerAction, start: bool) {
        for (kind,args) in action.map(&self.lowlevel) {
            self.lowlevel.send(kind,start,&args);
        }    
    }

    fn send_drag(&mut self, delta: (f64,f64), start: bool) {
          // XXX yuck, clones on critical path
        if self.hold {
            self.emit(&PointerAction::RunningHold(self.modifiers.clone(),delta),start);
        } else {
            self.emit(&PointerAction::RunningDrag(self.modifiers.clone(),delta),start);
        }
    }

    fn check_dragged(&mut self, config: &PointerConfig, current: &(f64,f64)) {
        if !self.dragged {
            let total_distance = (current.0-self.start_pos.0,current.1-self.start_pos.1);
            if total_distance.0.abs() + total_distance.1.abs() > config.click_radius {
                self.dragged = true;
                if !self.hold {
                    self.cursor = Some(self.lowlevel.set_cursor(&CursorCircumstance::Drag));
                }
            }
        }
    }

    fn drag_continue(&mut self, config: &PointerConfig, current: &(f64,f64)) {
        self.check_dragged(config,current);
        let delta = (current.0-self.prev_pos.0,current.1-self.prev_pos.1);
        self.send_drag(delta,true);
        self.prev_pos = *current;
    }

    fn drag_finished(&mut self, config: &PointerConfig, current: &(f64,f64)) -> bool {
        self.check_dragged(config,current);
        let delta = (current.0-self.prev_pos.0,current.1-self.prev_pos.1);
        self.send_drag(delta,true);
        self.alive = false;
        if self.dragged {
            self.send_drag((0.,0.),false);
            if self.hold {
                self.emit(&PointerAction::HoldDrag(self.modifiers.clone(),(current.0-self.start_pos.0,current.1-self.start_pos.1)),true);
            } else {
                self.emit(&PointerAction::Drag(self.modifiers.clone(),(current.0-self.start_pos.0,current.1-self.start_pos.1)),true);
            }
        }  
        self.cursor = None;
        self.dragged
    }
}

pub struct DragState(Arc<Mutex<DragStateData>>);

impl DragState {
    pub(super) fn new(config: &PointerConfig, lowlevel: &LowLevelState, current: &(f64,f64)) -> DragState {
        let inner = Arc::new(Mutex::new(DragStateData::new(lowlevel,current)));
        let inner2 = inner.clone();
        let hold_time = config.hold_delay;
        let drag_cursor_delay = config.drag_cursor_delay;
        lowlevel.timer(hold_time, move || {
            inner2.lock().unwrap().hold_timer_expired();
        });
        let inner2 = inner.clone();
        lowlevel.timer(drag_cursor_delay, move || {
            inner2.lock().unwrap().click_timer_expired();
        });
        DragState(inner)
    }

    pub(super) fn drag_continue(&mut self, config: &PointerConfig, current: &(f64,f64)) {
        self.0.lock().unwrap().drag_continue(config,current);
    }

    pub(super) fn drag_finished(&mut self, config: &PointerConfig, current: &(f64,f64)) -> bool {
        self.0.lock().unwrap().drag_finished(config,current)
    }
}
