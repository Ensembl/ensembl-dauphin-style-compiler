use std::sync::{ Arc, Mutex };
use crate::input::low::lowlevel::LowLevelState;
use crate::input::low::lowlevel::Modifiers;
use super::pointer::{ PointerConfig, PointerAction };
use super::cursor::CursorHandle;
use crate::run::CursorCircumstance;

struct FingerDrag {
    start_pos: (f64,f64),
    prev_pos: (f64,f64),
}

impl FingerDrag {
    fn new(pos: (f64,f64)) -> FingerDrag {
        FingerDrag {
            start_pos: pos,
            prev_pos: pos
        }
    }

    fn start(&self) -> (f64,f64) { self.start_pos }

    fn total_delta(&self, position: (f64,f64)) -> (f64,f64) {
        (position.0-self.start_pos.0,position.1-self.start_pos.1)
    }

    fn total_distance(&self, position: (f64,f64)) -> f64 {
        let total_delta = self.total_delta(position);
        total_delta.0.abs() + total_delta.1.abs()
    }

    fn delta(&mut self, position: (f64,f64)) -> (f64,f64) {
        let delta = (position.0-self.prev_pos.0,position.1-self.prev_pos.1);
        self.prev_pos = position;
        delta
    }
}

#[derive(Clone,PartialEq,Eq)]
enum DragMode {
    Unknown,
    Drag,
    Hold,
    Pinch
}

impl DragMode {
    fn cursor(&self) -> CursorCircumstance {
        match self {
            DragMode::Unknown => CursorCircumstance::Drag,
            DragMode::Drag => CursorCircumstance::Drag,
            DragMode::Hold => CursorCircumstance::Hold,
            DragMode::Pinch => CursorCircumstance::Pinch,
        }
    }
}

pub struct DragStateData {
    lowlevel: LowLevelState,
    modifiers: Modifiers,
    primary: FingerDrag,
    secondary: Option<FingerDrag>,
    mode: DragMode,
    alive: bool,
    #[allow(unused)] // keep as guard
    cursor: Option<CursorHandle>
}

impl DragStateData {
    fn new(lowlevel: &LowLevelState, primary: (f64,f64), secondary: Option<(f64,f64)>) -> DragStateData {
        let mut out = DragStateData {
            lowlevel: lowlevel.clone(),
            modifiers: lowlevel.modifiers(),
            primary: FingerDrag::new(primary),
            secondary: None,
            mode: DragMode::Unknown,
            alive: true,
            cursor: None
        };
        out.check_secondary(secondary);
        out
    }

    fn check_secondary(&mut self, secondary: Option<(f64,f64)>) {
        if let Some(secondary) = secondary {
            if self.secondary.is_none() {
                self.set_mode(DragMode::Pinch);
                self.secondary = Some(FingerDrag::new(secondary));
                self.emit(&PointerAction::SwitchToPinch(self.modifiers.clone(),self.primary.start(),secondary),true);
            }
        }
    }

    fn delta_secondary(&mut self, secondary: Option<(f64,f64)>) -> Option<(f64,f64)> {
        match (secondary,&mut self.secondary) {
            (Some(new),Some(finger)) => {
                Some(finger.delta(new))
            },
            _ => { None }
        }
    }

    fn total_delta_secondary(&mut self, secondary: Option<(f64,f64)>) -> Option<(f64,f64)> {
        match (secondary,&mut self.secondary) {
            (Some(new),Some(finger)) => {
                Some(finger.total_delta(new))
            },
            _ => { None }
        }
    }

    fn set_mode(&mut self, mode: DragMode) {
        self.mode = mode;
        self.cursor = Some(self.lowlevel.set_cursor(&self.mode.cursor()));
    }

    fn emit(&mut self, action: &PointerAction, start: bool) {
        for (kind,args) in action.map(&self.lowlevel) {
            self.lowlevel.send(kind,start,&args);
        }    
    }

    fn click_timer_expired(&mut self) {
        if !self.alive { return; }
        self.set_mode(self.mode.clone()); // Force cursor to be correct
    }

    fn hold_timer_expired(&mut self) {
        if !self.alive { return; }
        if self.mode == DragMode::Unknown {
            self.set_mode(DragMode::Hold);
            self.emit(&PointerAction::SwitchToHold(self.modifiers.clone(),self.primary.start()),true);
        }
    }

    fn send_drag(&mut self, delta: (f64,f64), secondary: Option<(f64,f64)>, start: bool) {
        // XXX yuck, clones on critical path
        match self.mode {
            DragMode::Drag | DragMode::Unknown => {
                self.emit(&PointerAction::RunningDrag(self.modifiers.clone(),delta),start);
            },
            DragMode::Hold => {
                self.emit(&PointerAction::RunningHold(self.modifiers.clone(),delta),start);
            },
            DragMode::Pinch => {
                if let Some(secondary) = secondary {
                    self.emit(&PointerAction::RunningPinch(self.modifiers.clone(),delta,secondary),start);
                }
            }
        }
    }

    fn check_dragged(&mut self, config: &PointerConfig, primary: (f64,f64)) {
        if self.mode == DragMode::Unknown {
            if self.primary.total_distance(primary) > config.click_radius {
                self.set_mode(DragMode::Drag);
            }
        }
    }

    fn drag_continue(&mut self, config: &PointerConfig, primary: (f64,f64), secondary: Option<(f64,f64)>) {
        self.check_secondary(secondary);
        self.check_dragged(config,primary);
        let delta_p = self.primary.delta(primary);
        let delta_s = self.delta_secondary(secondary);
        self.send_drag(delta_p,delta_s,true);
    }

    fn drag_finished(&mut self, config: &PointerConfig, primary: (f64,f64), secondary: Option<(f64,f64)>) -> bool {
        self.check_secondary(secondary);
        self.check_dragged(config,primary);
        let delta = self.primary.delta(primary);
        self.send_drag(delta,secondary,true);
        self.alive = false;
        self.cursor = None;
        let total_delta = self.primary.total_delta(primary);
        let total_delta_secondary = self.total_delta_secondary(secondary);
        match self.mode {
            DragMode::Unknown => { false },
            DragMode::Drag => {
                self.send_drag((0.,0.),None,false);
                self.emit(&PointerAction::Drag(self.modifiers.clone(),total_delta),true);
                true
            },
            DragMode::Hold => {
                self.send_drag((0.,0.),None,false);
                self.emit(&PointerAction::HoldDrag(self.modifiers.clone(),total_delta),true);
                true
            },
            DragMode::Pinch => {
                self.send_drag((0.,0.),None,false);
                if let Some(total_delta_secondary) = total_delta_secondary {
                    self.emit(&PointerAction::PinchDrag(self.modifiers.clone(),total_delta,total_delta_secondary),true);
                }
                true
            },
        }
    }
}

pub struct DragState(Arc<Mutex<DragStateData>>);

impl DragState {
    pub(super) fn new(config: &PointerConfig, lowlevel: &LowLevelState, primary: (f64,f64), secondary: Option<(f64,f64)>) -> DragState {
        let inner = Arc::new(Mutex::new(DragStateData::new(lowlevel,primary,secondary)));
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

    pub(super) fn drag_continue(&mut self, config: &PointerConfig, primary: (f64,f64), secondary: Option<(f64,f64)>) {
        self.0.lock().unwrap().drag_continue(config,primary,secondary);
    }

    pub(super) fn drag_finished(&mut self, config: &PointerConfig, primary: (f64,f64), secondary: Option<(f64,f64)>) -> bool {
        self.0.lock().unwrap().drag_finished(config,primary,secondary)
    }
}
