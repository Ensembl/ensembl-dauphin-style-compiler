use crate::util::error::{ confused_browser, confused_browser_option };
use std::collections::{ HashMap, HashSet };
use std::sync::{ Arc, Mutex };
use crate::PeregrineDom;
use wasm_bindgen::prelude::*;
use web_sys::{ MouseEvent, HtmlElement, Event, WheelEvent, window, Window };
use crate::input::{ InputEvent, InputEventKind, Distributor };
use crate::util::{ Message };
use super::event::{ EventSystem };
use super::mapping::InputMap;
use crate::run::PgPeregrineConfig;
use crate::run::{ PgConfigKey };
use super::lowlevel::Modifiers;
use js_sys::Date;
use wasm_bindgen::JsValue;
use std::convert::{ TryInto, TryFrom };
use wasm_bindgen::JsCast;

#[derive(Clone,Debug)]
enum MouseEventKind {
    Up,
    Down,
    Move
}

pub enum StateMachine {
    MouseUp,
    DragInProgress(Modifiers,(f64,f64),(f64,f64))
}


#[derive(Debug)]
pub enum MouseAction {
    RunningDrag(Modifiers,(f64,f64)),
    TotalDrag(Modifiers,(f64,f64)),
    Wheel(Modifiers,f64,(f64,f64))
}

impl MouseAction {
    pub fn map(&self, map: &InputMap) -> Vec<(InputEventKind,Vec<f64>)> {
        let mut out = vec![];
        let (kinds,modifiers) = match self {
            MouseAction::RunningDrag(modifiers,amount) => (vec![("RunningDrag",vec![amount.0,amount.1]),("MirrorRunningDrag",vec![-amount.0,-amount.1])],modifiers),
            MouseAction::TotalDrag(modifiers,amount) => (vec![("TotalDrag",vec![amount.0,amount.1])],modifiers),
            MouseAction::Wheel(modifiers,amount,pos) => (vec![("Wheel",vec![*amount,pos.0,pos.1])],modifiers)
        };
        for (name,args) in kinds {
            if let Some((action,map_args)) = map.map(&name,&modifiers) {
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
    fn process_event(&mut self, current: &(f64,f64), kind: &MouseEventKind, modifiers: &Arc<Mutex<Modifiers>>) -> Vec<MouseAction> {
        let mut new_state = None;
        let mut emit = vec![];
        match (&self,kind) {
            (StateMachine::MouseUp,MouseEventKind::Down) => {
                new_state = Some(StateMachine::DragInProgress(modifiers.lock().unwrap().clone(),*current,*current));
            },
            (StateMachine::DragInProgress(modifiers,start,prev),MouseEventKind::Move) => {
                let delta = (current.0-prev.0,current.1-prev.1);
                emit.push(MouseAction::RunningDrag(modifiers.clone(),delta));
                new_state = Some(StateMachine::DragInProgress(modifiers.clone(),*start,*current)); // XXX yuck, clone on critical path
            },
            (StateMachine::DragInProgress(modifiers,start,prev),MouseEventKind::Up) => {
                let delta = (current.0-prev.0,current.1-prev.1);
                let total = (current.0-start.0,current.1-start.1);
                emit.push(MouseAction::RunningDrag(modifiers.clone(),delta));
                emit.push(MouseAction::TotalDrag(modifiers.clone(),total));
                new_state = Some(StateMachine::MouseUp);
            },
            _ => {}
        }
        if let Some(state) = new_state.take() {
            *self = state;
        }
        emit
    }
}

pub(super) struct MouseEventHandler {
    state: StateMachine,
    canvas: HtmlElement,
    position: (f64,f64),
    distributor: Distributor<InputEvent>,
    mapping: InputMap,
    modifiers: Arc<Mutex<Modifiers>>
}

impl MouseEventHandler {
    fn new(distributor: &Distributor<InputEvent>, mapping: &InputMap, canvas: &HtmlElement, modifiers: &Arc<Mutex<Modifiers>>) -> MouseEventHandler {
        MouseEventHandler {
            state: StateMachine::MouseUp,
            canvas: canvas.clone(),
            position: (0.,0.),
            distributor: distributor.clone(),
            mapping: mapping.clone(),
            modifiers: modifiers.clone()
        }
    }

    fn abandon(&mut self, _event: &Event) {
        self.state.process_event(&self.position,&MouseEventKind::Up,&self.modifiers);
    }

    fn mouse_event(&mut self, kind: &MouseEventKind, event: &MouseEvent) {
        let rect = self.canvas.get_bounding_client_rect();
        let x = (event.client_x() as f64) - rect.left();
        let y = (event.client_y() as f64) - rect.top();
        self.position = (x,y);
        for action in self.state.process_event(&self.position,kind,&self.modifiers) {
            for (kind,args) in action.map(&self.mapping).drain(..) {
                self.distributor.send(InputEvent {
                    details: kind,
                    start: true,
                    amount: args,
                    timestamp_ms: Date::now()
                }); 
            }
        }
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
        for (kind,args) in MouseAction::Wheel(self.modifiers.lock().unwrap().clone(),amount,pos).map(&self.mapping) {
            self.distributor.send(InputEvent {
                details: kind,
                start: true,
                amount: args,
                timestamp_ms: Date::now()
            }); 
        }
        event.stop_propagation();
        event.prevent_default();
    }
}

pub(super) fn mouse_events(distributor: &Distributor<InputEvent>, dom: &PeregrineDom, mapping: &InputMap, modifiers: &Arc<Mutex<Modifiers>>) -> Result<EventSystem<MouseEventHandler>,Message> {
    let body = dom.body();
    let canvas = dom.canvas();
    let mut events = EventSystem::new(MouseEventHandler::new(distributor,mapping,dom.canvas(),modifiers));
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
    events.add(dom.canvas_frame(),"scroll",|handler,event: &Event| {
    })?;
    Ok(events)
}
