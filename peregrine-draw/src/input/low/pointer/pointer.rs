use std::sync::{ Arc, Mutex };
use crate::{input::low::modifiers::Modifiers, run::CursorCircumstance, util::{ Message }};
use crate::util::monostable::Monostable;
use crate::input::low::lowlevel::{ LowLevelState };
use js_sys::Date;
use super::{drag::DragState };
use crate::run::{ PgConfigKey, PgPeregrineConfig };
use crate::input::InputEventKind;
use super::cursor::CursorHandle;
use super::pinch::ScreenPosition;

pub(crate) struct PointerConfig {
    pub drag_cursor_delay: f64, // ms
    pub click_radius: f64, // px
    pub hold_delay: f64, // ms
    pub multiclick_time: f64, // ms
    pub wheel_timeout: f64, // ms
    pub pinch_min_sep: f64, // px
    pub pinch_min_scale: f64, // factor
    pub wheel_sensitivity: f64, // factor
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
        })
    }
}

pub(super) enum PointerAction {
    RunningDrag(Modifiers,(f64,f64)),
    RunningHold(Modifiers,(f64,f64)),
    RunningPinch(Modifiers,ScreenPosition),
    Drag(Modifiers,(f64,f64)),
    Wheel(Modifiers,f64,(f64,f64)),
    Click(Modifiers,(f64,f64)),
    DoubleClick(Modifiers,(f64,f64)),
    SwitchToPinch(Modifiers,ScreenPosition),
    SwitchToHold(Modifiers,(f64,f64)),
    HoldDrag(Modifiers,f64,f64,f64),
    PinchDrag(Modifiers,ScreenPosition),
}

impl PointerAction {
    pub fn map(&self, state: &LowLevelState) -> Vec<(InputEventKind,Vec<f64>)> {
        let mut out = vec![];
        let (kinds,modifiers) = match self {
            PointerAction::RunningDrag(modifiers,amount) => (vec![("RunningDrag",vec![amount.0,amount.1]),("MirrorRunningDrag",vec![-amount.0,-amount.1])],modifiers),
            PointerAction::RunningHold(modifiers,amount) => (vec![("RunningHold",vec![amount.0,amount.1]),("MirrorRunningHold",vec![-amount.0,-amount.1])],modifiers),
            PointerAction::RunningPinch(modifiers,pinch) => (
                vec![("RunningPinch",pinch.parameters())],modifiers
            ),
            PointerAction::Drag(modifiers,amount) => (vec![("Drag",vec![amount.0,amount.1])],modifiers),
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

pub struct Pointer {
    previous_click: Option<RecentClick>,
    start: (f64,f64),
    modifiers: Modifiers,
    drag: Option<DragState>,
    wheel_monostable: Monostable,
    wheel_cursor: Arc<Mutex<Option<(CursorHandle,CursorCircumstance)>>>
}

impl Pointer {
    pub(crate) fn new(lowlevel: &LowLevelState, config: &PointerConfig) -> Pointer {
        let wheel_cursor = Arc::new(Mutex::new(None));
        let wheel_cursor2 = wheel_cursor.clone();
        Pointer {
            drag: None,
            previous_click: None,
            start: (0.,0.),
            modifiers: lowlevel.modifiers(),
            wheel_cursor,
            wheel_monostable: Monostable::new(lowlevel.commander(), config.wheel_timeout, move || {
                wheel_cursor2.lock().unwrap().take();
            }),
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

    pub(crate) fn process_event(&mut self, config: &PointerConfig, lowlevel: &LowLevelState, primary: (f64,f64), secondary: Option<(f64,f64)>, kind: &PointerEventKind) -> Result<(),Message> {
        match (&mut self.drag,kind) {
            (None,PointerEventKind::Down) => {
                self.drag = Some(DragState::new(config,lowlevel,primary,secondary)?);
                self.start = primary;
                self.modifiers = lowlevel.modifiers();
            },
            (Some(drag_state),PointerEventKind::Move) => {
                drag_state.drag_continue(config,primary,secondary)?;
            },
            (Some(drag_state),PointerEventKind::Up) => {
                if !drag_state.drag_finished(config,primary,secondary)? {
                    self.click(config,lowlevel);
                }
                self.drag = None;
            },
            _ => {}
        }
        Ok(())
    }

    pub(crate) fn wheel_event(&mut self, lowlevel: &LowLevelState, position: &(f64,f64), amount: f64) {
        let circ = if amount > 0. { CursorCircumstance::WheelPositive } else { CursorCircumstance::WheelNegative };
        let mut cursor = self.wheel_cursor.lock().unwrap();
        match cursor.as_ref() {
            Some((_,x)) if x == &circ => {},
            _ => { *cursor = Some((lowlevel.set_cursor(&circ),circ)) }
        };
        self.wheel_monostable.set();
        for (kind,args) in PointerAction::Wheel(lowlevel.modifiers(),amount,*position).map(lowlevel) {
            lowlevel.send(kind,true,&args);
        }
    }
}
