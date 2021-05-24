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
use super::drag::DragState;

pub(super) struct MouseConfig {
    pub click_radius: f64, // px
    pub hold_delay: f64, // ms
    pub multiclick_time: f64
}

impl MouseConfig {
    fn new(config: &PgPeregrineConfig) -> Result<MouseConfig,Message> {
        Ok(MouseConfig {
            click_radius: config.get_f64(&PgConfigKey::MouseClickRadius)?,
            hold_delay: config.get_f64(&PgConfigKey::MouseHoldDwell)?,
            multiclick_time: config.get_f64(&PgConfigKey::DoubleClickTime)?
        })
    }
}

#[derive(Clone,Debug)]
enum MouseEventKind {
    Up,
    Down,
    Move
}

struct RecentClick {
    position: (f64,f64),
    time: f64
}

pub struct StateMachine {
    previous_click: Option<RecentClick>,
    start: (f64,f64),
    modifiers: Modifiers,
    drag: Option<DragState>
}


#[derive(Debug)]
pub enum MouseAction {
    RunningDrag(Modifiers,(f64,f64)),
    RunningHold(Modifiers,(f64,f64)),
    Drag(Modifiers,(f64,f64)),
    Wheel(Modifiers,f64,(f64,f64)),
    Click(Modifiers,(f64,f64)),
    DoubleClick(Modifiers,(f64,f64)),
    SwitchToHold(Modifiers,(f64,f64)),
    HoldDrag(Modifiers,(f64,f64)),
}

impl MouseAction {
    pub fn map(&self, state: &LowLevelState) -> Vec<(InputEventKind,Vec<f64>)> {
        let mut out = vec![];
        let (kinds,modifiers) = match self {
            MouseAction::RunningDrag(modifiers,amount) => (vec![("RunningDrag",vec![amount.0,amount.1]),("MirrorRunningDrag",vec![-amount.0,-amount.1])],modifiers),
            MouseAction::RunningHold(modifiers,amount) => (vec![("RunningHold",vec![amount.0,amount.1]),("MirrorRunningHold",vec![-amount.0,-amount.1])],modifiers),
            MouseAction::Drag(modifiers,amount) => (vec![("Drag",vec![amount.0,amount.1])],modifiers),
            MouseAction::Wheel(modifiers,amount,pos) => (vec![("Wheel",vec![*amount,pos.0,pos.1]),("MirrorWheel",vec![-*amount,pos.0,pos.1])],modifiers),
            MouseAction::Click(modifiers,pos) => (vec![("Click",vec![pos.0,pos.1])],modifiers),
            MouseAction::DoubleClick(modifiers,pos) => (vec![("DoubleClick",vec![pos.0,pos.1])],modifiers),
            MouseAction::SwitchToHold(modifiers,pos) => (vec![("SwitchToHold",vec![pos.0,pos.1])],modifiers),
            MouseAction::HoldDrag(modifiers,amount) => (vec![("Hold",vec![amount.0,amount.1])],modifiers),
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

impl StateMachine {
    fn new() -> StateMachine {
        StateMachine {
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

    fn send(&self, action: &MouseAction, lowlevel: &LowLevelState) {
        for (kind,args) in action.map(lowlevel) {
            lowlevel.send(kind,true,&args);
        }
    }

    fn check_double(&mut self, config: &MouseConfig) -> bool {
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

    fn click(&mut self, config: &MouseConfig, lowlevel: &LowLevelState) {
        self.send(&MouseAction::Click(self.modifiers.clone(),self.start),lowlevel);
        if self.check_double(config) {
            self.send(&MouseAction::DoubleClick(self.modifiers.clone(),self.start),lowlevel);
        }
    }

    fn process_event(&mut self, config: &MouseConfig, lowlevel: &LowLevelState, current: &(f64,f64), kind: &MouseEventKind) {
        match (&mut self.drag,kind) {
            (None,MouseEventKind::Down) => {
                self.drag = Some(DragState::new(config,lowlevel,current));
                self.start = *current;
                self.modifiers = lowlevel.modifiers();
            },
            (Some(drag_state),MouseEventKind::Move) => {
                drag_state.drag_continue(config,current);
            },
            (Some(drag_state),MouseEventKind::Up) => {
                if !drag_state.drag_finished(config,current) {
                    self.click(config,lowlevel);
                }
                self.drag = None;
            },
            _ => {}
        }
    }
}

pub(super) struct MouseEventHandler {
    state: StateMachine,
    lowlevel: LowLevelState,
    position: (f64,f64),
    config: Arc<MouseConfig>,
}

impl MouseEventHandler {
    fn new(config: Arc<MouseConfig>, lowlevel: &LowLevelState) -> MouseEventHandler {
        MouseEventHandler {
            state: StateMachine::new(),
            lowlevel: lowlevel.clone(),
            position: (0.,0.),
            config
        }
    }

    fn abandon(&mut self, _event: &Event) {
        self.state.process_event(&self.config,&self.lowlevel,&self.position,&MouseEventKind::Up);
    }

    fn mouse_event(&mut self, kind: &MouseEventKind, event: &MouseEvent) {
        let rect = self.lowlevel.dom().canvas_frame().get_bounding_client_rect();
        let x = (event.client_x() as f64) - rect.left();
        let y = (event.client_y() as f64) - rect.top();
        self.position = (x,y);
        self.state.process_event(&self.config,&self.lowlevel,&self.position,kind);
        match kind {
            MouseEventKind::Move => {},
            _ => { event.stop_propagation(); event.prevent_default(); }
        }
    }

    fn wheel_amount(&self, event: &WheelEvent) -> f64 {
        let mode = event.delta_mode();
        let y = event.delta_y();
        match mode {
            0 => y,
            1 => y*40.,
            _ => y*800.
        }
    }

    fn wheel_event(&mut self, event: &WheelEvent) {
        let amount = self.wheel_amount(event);
        let pos = self.position;
        for (kind,args) in MouseAction::Wheel(self.lowlevel.modifiers(),amount,pos).map(&self.lowlevel) {
            self.lowlevel.send(kind,true,&args);
        }
        event.stop_propagation();
        event.prevent_default();
    }
}

pub(super) fn mouse_events(config: &PgPeregrineConfig, state: &LowLevelState) -> Result<EventSystem<MouseEventHandler>,Message> {
    let mouse_config = Arc::new(MouseConfig::new(config)?);
    let dom = state.dom();
    let body = dom.body();
    let canvas = dom.canvas();
    let mut events = EventSystem::new(MouseEventHandler::new(mouse_config,state));
    events.add(canvas,"mousedown", |handler,event| {
        handler.mouse_event(&MouseEventKind::Down,event)
    })?;
    events.add(canvas,"mouseup", |handler,event| {
        handler.mouse_event(&MouseEventKind::Up,event)
    })?;
    events.add(body,"mousemove", |handler,event| {
        handler.mouse_event(&MouseEventKind::Move,event)
    })?;
    events.add(canvas,"wheel", |handler,event| {
        handler.wheel_event(event)
    })?;
    events.add(body,"mouseleave",|handler,event| {
        handler.abandon(&event)
    })?;
    events.add(dom.canvas_frame(),"scroll",|_,_: &Event| {
    })?;
    Ok(events)
}
