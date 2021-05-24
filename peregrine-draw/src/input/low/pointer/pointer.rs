use crate::util::{ Message };
use crate::input::low::lowlevel::{ LowLevelState, Modifiers };
use js_sys::Date;
use super::drag::DragState;
use crate::run::{ PgConfigKey, PgPeregrineConfig };
use crate::input::InputEventKind;

pub(crate) struct PointerConfig {
    pub click_radius: f64, // px
    pub hold_delay: f64, // ms
    pub multiclick_time: f64 // ms
}

impl PointerConfig {
    pub fn new(config: &PgPeregrineConfig) -> Result<PointerConfig,Message> {
        Ok(PointerConfig {
            click_radius: config.get_f64(&PgConfigKey::MouseClickRadius)?,
            hold_delay: config.get_f64(&PgConfigKey::MouseHoldDwell)?,
            multiclick_time: config.get_f64(&PgConfigKey::DoubleClickTime)?
        })
    }
}

#[derive(Debug)]
pub(super) enum PointerAction {
    RunningDrag(Modifiers,(f64,f64)),
    RunningHold(Modifiers,(f64,f64)),
    Drag(Modifiers,(f64,f64)),
    Wheel(Modifiers,f64,(f64,f64)),
    Click(Modifiers,(f64,f64)),
    DoubleClick(Modifiers,(f64,f64)),
    SwitchToHold(Modifiers,(f64,f64)),
    HoldDrag(Modifiers,(f64,f64)),
}

impl PointerAction {
    pub fn map(&self, state: &LowLevelState) -> Vec<(InputEventKind,Vec<f64>)> {
        let mut out = vec![];
        let (kinds,modifiers) = match self {
            PointerAction::RunningDrag(modifiers,amount) => (vec![("RunningDrag",vec![amount.0,amount.1]),("MirrorRunningDrag",vec![-amount.0,-amount.1])],modifiers),
            PointerAction::RunningHold(modifiers,amount) => (vec![("RunningHold",vec![amount.0,amount.1]),("MirrorRunningHold",vec![-amount.0,-amount.1])],modifiers),
            PointerAction::Drag(modifiers,amount) => (vec![("Drag",vec![amount.0,amount.1])],modifiers),
            PointerAction::Wheel(modifiers,amount,pos) => (vec![("Wheel",vec![*amount,pos.0,pos.1]),("MirrorWheel",vec![-*amount,pos.0,pos.1])],modifiers),
            PointerAction::Click(modifiers,pos) => (vec![("Click",vec![pos.0,pos.1])],modifiers),
            PointerAction::DoubleClick(modifiers,pos) => (vec![("DoubleClick",vec![pos.0,pos.1])],modifiers),
            PointerAction::SwitchToHold(modifiers,pos) => (vec![("SwitchToHold",vec![pos.0,pos.1])],modifiers),
            PointerAction::HoldDrag(modifiers,amount) => (vec![("Hold",vec![amount.0,amount.1])],modifiers),
        };
        for (name,args) in kinds {
            if let Some((action,map_args)) = state.map(&name,&modifiers) {
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

#[derive(Clone,Debug)]
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
    drag: Option<DragState>
}

impl Pointer {
    pub(crate) fn new() -> Pointer {
        Pointer {
            drag: None,
            previous_click: None,
            start: (0.,0.),
            modifiers: Modifiers { // XXX constructor
                shift: false,
                control: false,
                alt: false
            }
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

    pub(crate) fn process_event(&mut self, config: &PointerConfig, lowlevel: &LowLevelState, current: &(f64,f64), kind: &PointerEventKind) {
        match (&mut self.drag,kind) {
            (None,PointerEventKind::Down) => {
                self.drag = Some(DragState::new(config,lowlevel,current));
                self.start = *current;
                self.modifiers = lowlevel.modifiers();
            },
            (Some(drag_state),PointerEventKind::Move) => {
                drag_state.drag_continue(config,current);
            },
            (Some(drag_state),PointerEventKind::Up) => {
                if !drag_state.drag_finished(config,current) {
                    self.click(config,lowlevel);
                }
                self.drag = None;
            },
            _ => {}
        }
    }

    pub(crate) fn wheel_event(&mut self, lowlevel: &LowLevelState, position: &(f64,f64), amount: f64) {
        for (kind,args) in PointerAction::Wheel(lowlevel.modifiers(),amount,*position).map(lowlevel) {
            lowlevel.send(kind,true,&args);
        }
    }
}
