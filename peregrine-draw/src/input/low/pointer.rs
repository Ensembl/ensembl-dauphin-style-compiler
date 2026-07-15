use std::sync::{ Arc, Mutex };
use crate::{input::low::{modifiers::Modifiers}, run::CursorCircumstance, util::{ Message }, webgl::global::WebGlGlobal};
use crate::util::monostable::Monostable;
use crate::input::low::lowlevel::{ LowLevelState };
use js_sys::Date;
use peregrine_toolkit::{plumbing::oneshot::OneShot};
use crate::run::{ PgConfigKey, PgPeregrineConfig };
use crate::input::InputEventKind;

use super::gesture::{core::{gesture::Gesture, cursor::CursorHandle}, node::pinch::ScreenPosition};

pub(crate) struct PointerConfig {
    pub drag_cursor_delay: f64, // ms
    pub click_radius: f64, // px
    pub hold_delay: f64, // ms
    pub multiclick_time: f64, // ms
    pub wheel_timeout: f64, // ms
    pub pinch_min_sep: f64, // px
    pub pinch_min_scale: f64, // factor
    pub wheel_sensitivity: f64, // factor
    pub min_hold_drag_size: f64, // factor
    pub min_vert_odometer: f64, // px
    pub min_vert_numer: f64,
    pub min_vert_denom: f64
}

impl PointerConfig {
    pub fn new(config: &PgPeregrineConfig) -> Result<PointerConfig,Message> {
        Ok(PointerConfig {
            drag_cursor_delay: config.get_f64(&PgConfigKey::DragCursorDelay)?,
            click_radius: config.get_f64(&PgConfigKey::MouseClickRadius)?,
            hold_delay: config.get_f64(&PgConfigKey::MouseHoldDwell)?,
            multiclick_time: config.get_f64(&PgConfigKey::DoubleClickTime)?,
            wheel_timeout: config.get_f64(&PgConfigKey::WheelTimeout)?,
            pinch_min_sep: config.get_f64(&PgConfigKey::PinchMinSep)?,
            pinch_min_scale: config.get_f64(&PgConfigKey::PinchMinScale)?,
            wheel_sensitivity: config.get_f64(&PgConfigKey::WheelSensitivity)?,
            min_hold_drag_size: config.get_f64(&PgConfigKey::MinHoldDragSize)?,
            min_vert_odometer: config.get_f64(&PgConfigKey::MinVertOdometer)?,
            min_vert_numer: config.get_f64(&PgConfigKey::MinVertNumerator)?,
            min_vert_denom: config.get_f64(&PgConfigKey::MinVertDenominator)?,

        })
    }
}

pub(super) enum PointerAction {
    RunningDrag(Modifiers,(f64,f64)),
    VerticalDrag(Modifiers,(f64,f64)),
    RunningHold(Modifiers,(f64,f64)),
    RunningPinch(Modifiers,ScreenPosition),
    Drag(Modifiers,(f64,f64)),
    HorizontalWheel(Modifiers,f64),
    Wheel(Modifiers,f64,(f64,f64)),
    Click(Modifiers,(f64,f64)),
    DoubleClick(Modifiers,(f64,f64)),
    SwitchToPinch(Modifiers,ScreenPosition),
    SwitchToHold(Modifiers,(f64,f64)),
    HoldDrag(Modifiers,f64,f64,f64),
    PinchDrag(Modifiers,ScreenPosition),
}

impl PointerAction {
    fn map(&self, state: &LowLevelState) -> Vec<(InputEventKind,Vec<f64>)> {
        let mut out = vec![];
        let (kinds,modifiers) = match self {
            PointerAction::RunningDrag(modifiers,amount) => (vec![("RunningDrag",vec![amount.0,amount.1]),("MirrorRunningDrag",vec![-amount.0,-amount.1])],modifiers),
            PointerAction::VerticalDrag(modifiers,amount) => (vec![("VerticalDrag",vec![amount.0,amount.1]),("MirrorVerticalDrag",vec![-amount.0,-amount.1])],modifiers),
            PointerAction::RunningHold(modifiers,amount) => (vec![("RunningHold",vec![amount.0,amount.1]),("MirrorRunningHold",vec![-amount.0,-amount.1])],modifiers),
            PointerAction::RunningPinch(modifiers,pinch) => (
                vec![("RunningPinch",pinch.parameters())],modifiers
            ),
            PointerAction::Drag(modifiers,amount) => (vec![("Drag",vec![amount.0,amount.1])],modifiers),
            PointerAction::HorizontalWheel(modifiers,amount) => (vec![("HorizontalWheel",vec![*amount]),("MirrorHorizontalWheel",vec![-*amount])],modifiers),
            PointerAction::Wheel(modifiers,amount,pos) => (vec![("Wheel",vec![*amount,pos.0,pos.1]),("MirrorWheel",vec![-*amount,pos.0,pos.1])],modifiers),
            PointerAction::Click(modifiers,pos) => (vec![("Click",vec![pos.0,pos.1])],modifiers),
            PointerAction::DoubleClick(modifiers,pos) => (vec![("DoubleClick",vec![pos.0,pos.1])],modifiers),
            PointerAction::SwitchToPinch(modifiers,pinch) => (
                vec![("SwitchToPinch",pinch.parameters())],modifiers
            ),
            PointerAction::SwitchToHold(modifiers,pos) => (vec![("SwitchToHold",vec![pos.0,pos.1])],modifiers),
            PointerAction::HoldDrag(modifiers,scale,centre,y) => (vec![("Court",vec![*scale,*centre,*y])],modifiers),
            PointerAction::PinchDrag(modifiers,pinch) => (
                vec![("Pinch",pinch.parameters())],modifiers
            ),
        };
        for (name,args) in kinds {
            for (action,map_args) in state.map(&name,&modifiers) {
                let mut out_args = args.to_vec();
                for (i,arg) in map_args.iter().enumerate() {
                    if i < args.len() { out_args[i] = *arg; }
                }
                out.push((action,out_args));
            }
        }
        out
    }

    pub(super) fn emit(&self, lowlevel: &LowLevelState, start: bool) {
        for (kind,args) in self.map(lowlevel) {
            lowlevel.send(kind,start,&args);
        }
    }
}

#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) enum PointerEventKind {
    Up,
    Down,
    Move
}

struct RecentClick {
    position: (f64,f64),
    time: f64
}

pub(crate) struct Pointer {
    previous_click: Option<RecentClick>,
    start: (f64,f64),
    modifiers: Modifiers,
    drag: Option<Gesture>,
    wheel_monostable: Monostable,
    wheel_cursor: Arc<Mutex<Option<(CursorHandle,CursorCircumstance)>>>
}

impl Pointer {
    pub(crate) fn new(lowlevel: &LowLevelState, config: &PointerConfig, shutdown: &OneShot) -> Pointer {
        let wheel_cursor = Arc::new(Mutex::new(None));
        let wheel_cursor2 = wheel_cursor.clone();
        Pointer {
            drag: None,
            previous_click: None,
            start: (0.,0.),
            modifiers: lowlevel.modifiers(),
            wheel_cursor,
            wheel_monostable: Monostable::new(lowlevel.commander(), config.wheel_timeout,shutdown, move || {
                wheel_cursor2.lock().expect("failed to lock wheel cursor for reset").take();
            })
        }
    }

    fn send(&self, action: &PointerAction, lowlevel: &LowLevelState) {
        for (kind,args) in action.map(lowlevel) {
            lowlevel.send(kind,true,&args);
        }
    }

    fn check_double(&mut self, config: &PointerConfig) -> bool {
        let click = RecentClick {
            position: self.start.clone(),
            time: Date::now()
        };
        let mut double_click = false;
        if let Some(old_click) = self.previous_click.take() {
            let ago = click.time - old_click.time;
            let distance = (click.position.0-old_click.position.0).abs() + (click.position.1-old_click.position.1).abs();
            if ago < config.multiclick_time && distance < config.click_radius {
                double_click = true;
            }
        }
        self.previous_click = Some(click);
        double_click
    }

    fn click(&mut self, config: &PointerConfig, lowlevel: &LowLevelState) {
        self.send(&PointerAction::Click(self.modifiers.clone(),self.start),lowlevel);
        if self.check_double(config) {
            self.send(&PointerAction::DoubleClick(self.modifiers.clone(),self.start),lowlevel);
        }
    }

    fn set_wheel_cursor(&mut self, lowlevel: &LowLevelState, circumstance: Option<CursorCircumstance>) {
        let mut cursor = self.wheel_cursor.lock().expect("failed to lock wheel cursor for update");
        if let Some(circ) = circumstance {
            match cursor.as_ref() {
                Some((_,current)) if current == &circ => {},
                _ => { *cursor = Some((lowlevel.set_cursor(&circ),circ)); }
            };
            self.wheel_monostable.set();
        } else {
            cursor.take();
        }
    }

    pub(crate) fn process_event(&mut self, config: &Arc<PointerConfig>, lowlevel: &LowLevelState, gl: &Arc<Mutex<WebGlGlobal>>, primary: (f64,f64), secondary: Option<(f64,f64)>, kind: &PointerEventKind) -> Result<(),Message> {
        match (&mut self.drag,kind) {
            (None,PointerEventKind::Down) => {
                self.drag = Some(Gesture::new(config,lowlevel,gl,primary,secondary,lowlevel.target_reporter())?);
                self.start = primary;
                self.modifiers = lowlevel.modifiers();
            },
            (Some(drag_state),PointerEventKind::Move) => {
                drag_state.drag_continue(primary,secondary)?;
            },
            (Some(drag_state),PointerEventKind::Up) => {
                if !drag_state.drag_finished(primary,secondary)? {
                    self.click(config,lowlevel);
                }
                self.drag = None;
            },
            _ => {}
        }
        Ok(())
    }

    pub(crate) fn wheel_event(&mut self, lowlevel: &LowLevelState, position: &(f64,f64), delta_x: f64, delta_y: f64) {
        if delta_x.abs() > delta_y.abs() {
            self.set_wheel_cursor(lowlevel,None);
            PointerAction::HorizontalWheel(lowlevel.modifiers(),delta_x).emit(lowlevel,true);
        } else {
            let circ = if delta_y > 0. { CursorCircumstance::WheelPositive } else { CursorCircumstance::WheelNegative };
            self.set_wheel_cursor(lowlevel,Some(circ));
            PointerAction::Wheel(lowlevel.modifiers(),delta_y/10.,*position).emit(lowlevel,true);
        }
    }
}
