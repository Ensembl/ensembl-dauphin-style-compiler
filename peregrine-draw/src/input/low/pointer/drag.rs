use std::sync::{ Arc, Mutex };
use crate::Message;
use crate::input::low::lowlevel::LowLevelState;
use crate::input::low::lowlevel::Modifiers;
use crate::input::low::spectre::MarchingAnts;
use super::pinch::PinchManager;
use super::pinch::PinchManagerFactory;
use super::pointer::{ PointerConfig, PointerAction };
use super::cursor::CursorHandle;
use super::super::spectre::Spectre;
use super::super::spectremanager::SpectreHandle;
use crate::run::CursorCircumstance;
use super::pinch::FingerAxis;

struct FingerDelta(FingerAxis,FingerAxis);

impl FingerDelta {
    fn new(position: (f64,f64)) -> FingerDelta {
        FingerDelta(FingerAxis::new(position.0),FingerAxis::new(position.1))
    }

    fn start(&self) -> (f64,f64) { (self.0.start(),self.1.start()) }
    fn current(&self) -> (f64,f64) { (self.0.current(),self.1.current()) }
    fn set(&mut self, position: (f64,f64)) { self.0.set(position.0); self.1.set(position.1); }
    fn reset(&mut self) { self.0.reset(); self.1.reset(); }
    fn delta(&self) -> (f64,f64) { (self.0.delta(),self.1.delta()) }
}

struct FingerDrag {
    overall: FingerDelta,
    incremental: FingerDelta
}

impl FingerDrag {
    fn new(pos: (f64,f64)) -> FingerDrag {
        FingerDrag {
            overall: FingerDelta::new(pos),
            incremental: FingerDelta::new(pos)
        }
    }

    fn start(&self) -> (f64,f64) { self.overall.start() }
    fn current(&self) -> (f64,f64) { self.overall.current() }

    fn total_delta(&self) -> (f64,f64) {
        self.overall.delta()
    }

    fn set(&mut self, position: (f64,f64)) {
        self.overall.set(position);
        self.incremental.set(position);
    }

    fn total_distance(&self) -> f64 {
        let total_delta = self.total_delta();
        total_delta.0.abs() + total_delta.1.abs()
    }

    fn delta(&mut self) -> (f64,f64) {
        let delta = self.incremental.delta();
        self.incremental.reset();
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
    pinch_manager_factory: PinchManagerFactory,
    pinch: Option<PinchManager>,
    mode: DragMode,
    alive: bool,
    #[allow(unused)] // keep as guard
    cursor: Option<CursorHandle>,
    #[allow(unused)] // keep as guard
    spectre: Option<SpectreHandle>
}

impl DragStateData {
    fn new(lowlevel: &LowLevelState, config: &PointerConfig, primary: (f64,f64), secondary: Option<(f64,f64)>) -> Result<DragStateData,Message> {
        let mut out = DragStateData {
            lowlevel: lowlevel.clone(),
            modifiers: lowlevel.modifiers(),
            primary: FingerDrag::new(primary),
            pinch_manager_factory: PinchManagerFactory::new(config),
            pinch: None,
            mode: DragMode::Unknown,
            alive: true,
            cursor: None,
            spectre: None
        };
        out.check_secondary(primary,secondary)?;
        Ok(out)
    }

    fn update_spectre(&self) {
        if let Some(spectre) = &self.spectre {
            spectre.update(Spectre::MarchingAnts(self.make_ants()));
        }
    }

    fn check_secondary(&mut self, primary: (f64,f64), secondary: Option<(f64,f64)>) -> Result<(),Message> {
        if let Some(secondary) = secondary {
            if self.pinch.is_none() {
                if let Some(stage) = self.lowlevel.stage() {
                    if let Some(pinch_manager) = self.pinch_manager_factory.create(&stage,primary,secondary)? {
                        self.set_mode(DragMode::Pinch);
                        let position = pinch_manager.position();
                        self.pinch = Some(pinch_manager);
                        self.emit(&PointerAction::SwitchToPinch(self.modifiers.clone(),position),true);
                    }
                }
            }
            if let Some(pinch) = &mut self.pinch {
                pinch.set_position(primary,secondary);
            }
        }
        Ok(())
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

    fn make_ants(&self) -> MarchingAnts {
        MarchingAnts::new((
            self.primary.start().1,
            self.primary.start().0,
            self.primary.current().1,
            self.primary.current().0
        ))
    }

    fn hold_timer_expired(&mut self) {
        if !self.alive { return; }
        if self.mode == DragMode::Unknown {
            self.set_mode(DragMode::Hold);
            self.spectre = Some(self.lowlevel.add_spectre(Spectre::MarchingAnts(self.make_ants())));
            self.emit(&PointerAction::SwitchToHold(self.modifiers.clone(),self.primary.start()),true);
        }
    }

    fn send_drag(&mut self, delta: (f64,f64), start: bool) {
        // XXX yuck, clones on critical path
        match self.mode {
            DragMode::Drag | DragMode::Unknown => {
                self.emit(&PointerAction::RunningDrag(self.modifiers.clone(),delta),start);
            },
            DragMode::Hold => {
                self.emit(&PointerAction::RunningHold(self.modifiers.clone(),delta),start);
            },
            DragMode::Pinch => {
                if let Some(pinch) = &self.pinch {
                    self.emit(&PointerAction::RunningPinch(self.modifiers.clone(),pinch.position()),start);
                }
            }
        }
    }

    fn check_dragged(&mut self, config: &PointerConfig) {
        if self.mode == DragMode::Unknown {
            if self.primary.total_distance() > config.click_radius {
                self.set_mode(DragMode::Drag);
            }
        }
    }

    fn drag_continue(&mut self, config: &PointerConfig, primary: (f64,f64), secondary: Option<(f64,f64)>) -> Result<(),Message> {
        self.primary.set(primary);
        self.check_secondary(primary,secondary)?;
        self.check_dragged(config);
        self.update_spectre();
        let delta_p = self.primary.delta();
        self.send_drag(delta_p,true);
        Ok(())
    }

    fn drag_finished(&mut self, config: &PointerConfig, primary: (f64,f64), secondary: Option<(f64,f64)>) -> Result<bool,Message> {
        self.primary.set(primary);
        self.check_secondary(primary,secondary)?;
        self.check_dragged(config);
        self.update_spectre();
        let delta = self.primary.delta();
        self.send_drag(delta,true);
        self.alive = false;
        self.cursor = None;
        let total_delta = self.primary.total_delta();
        Ok(match self.mode {
            DragMode::Unknown => { false },
            DragMode::Drag => {
                self.send_drag((0.,0.),false);
                self.emit(&PointerAction::Drag(self.modifiers.clone(),total_delta),true);
                true
            },
            DragMode::Hold => {
                self.send_drag((0.,0.),false);
                self.emit(&PointerAction::HoldDrag(self.modifiers.clone(),total_delta),true);
                true
            },
            DragMode::Pinch => {
                self.send_drag((0.,0.),false);
                if let Some(pinch) = &self.pinch {
                    self.emit(&PointerAction::PinchDrag(self.modifiers.clone(),pinch.position()),true);
                }
                true
            },
        })
    }
}

pub struct DragState(Arc<Mutex<DragStateData>>);

impl DragState {
    pub(super) fn new(config: &PointerConfig, lowlevel: &LowLevelState, primary: (f64,f64), secondary: Option<(f64,f64)>) -> Result<DragState,Message> {
        let inner = Arc::new(Mutex::new(DragStateData::new(lowlevel,config,primary,secondary)?));
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
        Ok(DragState(inner))
    }

    pub(super) fn drag_continue(&mut self, config: &PointerConfig, primary: (f64,f64), secondary: Option<(f64,f64)>) -> Result<(),Message> {
        self.0.lock().unwrap().drag_continue(config,primary,secondary)
    }

    pub(super) fn drag_finished(&mut self, config: &PointerConfig, primary: (f64,f64), secondary: Option<(f64,f64)>) -> Result<bool,Message> {
        self.0.lock().unwrap().drag_finished(config,primary,secondary)
    }
}
