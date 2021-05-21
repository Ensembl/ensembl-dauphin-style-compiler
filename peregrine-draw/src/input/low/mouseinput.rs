use std::collections::{ HashMap, HashSet };
use std::sync::{ Arc, Mutex };
use crate::PeregrineDom;
use wasm_bindgen::prelude::*;
use web_sys::{ MouseEvent, HtmlElement, Event };
use crate::input::{ InputEvent, InputEventKind, Distributor };
use crate::util::{ Message };
use super::event::{ add_event, remove_event, window_add_event, window_remove_event };
use super::mapping::InputMap;
use crate::run::PgPeregrineConfig;
use crate::run::{ PgConfigKey };
use super::lowlevel::Modifiers;
use js_sys::Date;

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
}

impl MouseAction {
    pub fn map(&self, map: &InputMap) -> Vec<(InputEventKind,Vec<f64>)> {
        let mut out = vec![];
        let (kinds,modifiers) = match self {
            MouseAction::RunningDrag(modifiers,amount) => (vec![("RunningDrag",vec![amount.0,amount.1]),("MirrorRunningDrag",vec![-amount.0,-amount.1])],modifiers),
            MouseAction::TotalDrag(modifiers,amount) => (vec![("TotalDrag",vec![amount.0,amount.1])],modifiers)
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

struct MouseEventHandler {
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

    fn abandon(&mut self, event: &Event) {
        use web_sys::console;
        console::log_1(&format!("abandon").into());
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
    }
}

fn make_event(handler: &Arc<Mutex<MouseEventHandler>>) -> Closure<dyn Fn(Event)> {
    let handler = handler.clone();
    Closure::wrap(Box::new(move |event: Event| {
        let handler = handler.clone();
        handler.lock().unwrap().abandon(&event);
    }) as Box<dyn Fn(Event)>)
}

// XXX factor with keyboard
fn make_mouse_event(kind: MouseEventKind, handler: &Arc<Mutex<MouseEventHandler>>) -> Closure<dyn Fn(MouseEvent)> {
    let handler = handler.clone();
    let kind = kind.clone();
    Closure::wrap(Box::new(move |event: MouseEvent| {
        let handler = handler.clone();
        handler.lock().unwrap().mouse_event(&kind,&event);
    }) as Box<dyn Fn(MouseEvent)>)
}

#[derive(Clone)]
pub struct MouseInput {
    down_closure: Arc<Closure<dyn Fn(MouseEvent) + 'static>>,
    up_closure: Arc<Closure<dyn Fn(MouseEvent) + 'static>>,
    move_closure: Arc<Closure<dyn Fn(MouseEvent) + 'static>>,
    blur_closure: Arc<Closure<dyn Fn(Event) + 'static>>,
    el: HtmlElement
}

impl MouseInput {
    pub fn new(distributor: &Distributor<InputEvent>, dom: &PeregrineDom, mapping: &InputMap, modifiers: &Arc<Mutex<Modifiers>>) -> Result<MouseInput,Message> {
        let body = dom.body();
        let handler = Arc::new(Mutex::new(MouseEventHandler::new(distributor,mapping,dom.canvas(),modifiers)));
        let up_closure = make_mouse_event(MouseEventKind::Up,&handler);
        let down_closure = make_mouse_event(MouseEventKind::Down,&handler);
        let move_closure = make_mouse_event(MouseEventKind::Move,&handler);
        let blur_closure = make_event(&handler);
        add_event(body,"mousedown",&down_closure)?;
        add_event(body,"mouseup",&up_closure)?;
        add_event(body,"mousemove",&move_closure)?;
        add_event(body,"mouseleave",&blur_closure)?;
        Ok(MouseInput {
            up_closure: Arc::new(up_closure),
            down_closure: Arc::new(down_closure),
            move_closure: Arc::new(move_closure),
            blur_closure: Arc::new(blur_closure),
            el: body.clone()
        })
    }

    pub fn finish(&self) -> Result<(),Message> { // XXX pub
        remove_event(&self.el,"mousedown",&self.down_closure.as_ref())?;
        remove_event(&self.el,"mouseup",&self.up_closure.as_ref())?;
        remove_event(&self.el,"mousemove",&self.move_closure.as_ref())?;
        remove_event(&self.el,"mouseleave",&self.blur_closure.as_ref())?;
        Ok(())
    }
}

impl Drop for MouseInput {
    fn drop(&mut self) {
        self.finish().ok();
    }
}
