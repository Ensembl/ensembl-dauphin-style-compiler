use std::sync::{Arc, Mutex};
use crate::webgl::global::WebGlGlobal;
use crate::{run::PgPeregrineConfig };
use peregrine_toolkit::lock;
use peregrine_toolkit::plumbing::oneshot::OneShot;
use peregrine_toolkit_async::sync::needed::Needed;
use web_sys::{Event, MouseEvent, PointerEvent, WheelEvent};
use crate::util::{ Message };
use crate::util::error::confused_browser;
use super::event::EventHandle;
use super::pointer::{PointerEventKind, Pointer, PointerConfig};
use super::{ lowlevel::LowLevelState };

fn position(lowlevel: &LowLevelState, event: &MouseEvent) -> (f64,f64) {
    let rect = lowlevel.dom().canvas().get_bounding_client_rect();
    let x = (event.client_x() as f64) - rect.left();
    let y = (event.client_y() as f64) - rect.top();
    (x,y)
}

struct Finger {
    position: (f64,f64),
    id: Option<i32>,
    downstream: bool
}

impl Finger {
    fn new() -> Finger {
        Finger {
            position: (0.,0.),
            id: None,
            downstream: false
        }
    }

    fn mine(&mut self, event: &PointerEvent, kind: &PointerEventKind) -> bool {
        let down = *kind == PointerEventKind::Down;
        let event_id = event.pointer_id();
        if let Some(id) = self.id {
            if event_id != id {
                if down { // down event for another finger, ie there are downstream fingers
                    self.downstream = true;
                }
                return false;
            }
        } else if down {
            self.id = Some(event_id);
        }
        true
    }

    fn update_position(&mut self, lowlevel: &LowLevelState, event: &PointerEvent, kind: &PointerEventKind) {
        self.position = position(lowlevel,event);
        if *kind == PointerEventKind::Up {
            self.id = None;
        }
    }

    fn position(&self) -> (f64,f64) { self.position }
    
    fn downstream(&mut self, kind: &PointerEventKind) -> bool { 
        let out = self.downstream;
        if *kind == PointerEventKind::Up {
            self.downstream = false;
        }
        out
    }
}

struct MouseEventHandlerState {
    pointer: Pointer,
    primary: Finger,
    secondary: Finger
}

#[derive(Clone)]
pub(super) struct MouseEventHandler {
    lowlevel: LowLevelState,
    state: Arc<Mutex<MouseEventHandlerState>>,
    config: Arc<PointerConfig>,
    gl: Arc<Mutex<WebGlGlobal>>
}

impl MouseEventHandler {
    fn new(config: Arc<PointerConfig>, lowlevel: &LowLevelState, gl: &Arc<Mutex<WebGlGlobal>>, shutdown: &OneShot) -> MouseEventHandler {
        MouseEventHandler {
            state: Arc::new(Mutex::new(MouseEventHandlerState { 
                pointer: Pointer::new(lowlevel,&config,shutdown),
                primary: Finger::new(),
                secondary: Finger::new(),
            })),
            lowlevel: lowlevel.clone(),
            config,
            gl: gl.clone()
        }
    }

    fn report(&mut self, kind: &PointerEventKind) {
        let mut state = lock!(self.state);
        let secondary = if state.primary.downstream(kind) {
            Some(state.secondary.position())
        } else {
            None
        };
        let primary = state.primary.position();
        // XXX handle errors
        state.pointer.process_event(&self.config,&self.lowlevel,&self.gl,primary,secondary,&kind);
    }

    fn abandon(&mut self, event: &PointerEvent) {
        let mut report_up = false;
        let mut state = lock!(self.state);
        if state.primary.mine(event,&PointerEventKind::Up) {
            state.primary.update_position(&self.lowlevel,event,&PointerEventKind::Up);
            report_up = true;
        }
        if state.secondary.mine(event,&PointerEventKind::Up) {
            state.secondary.update_position(&self.lowlevel,event,&PointerEventKind::Up);
        }
        drop(state);
        if report_up {
            self.report(&PointerEventKind::Up);
        }
    }

    fn mouse_event(&mut self, kind: &PointerEventKind, event: &PointerEvent, mouse_moved: &Needed) {
        mouse_moved.set();
        self.lowlevel.set_pointer_last_seen(position(&self.lowlevel,event));
        let mut reported_kind = Some(kind.clone());
        let mut state = lock!(self.state);
        if state.primary.mine(event,kind) {
            state.primary.update_position(&self.lowlevel,event,kind);
        } else if state.secondary.mine(event,kind) {
            state.secondary.update_position(&self.lowlevel,event,kind);
            reported_kind = Some(PointerEventKind::Move);
        } else {
            reported_kind = None;
        }
        drop(state);
        if let Some(kind) = reported_kind {
            self.report(&kind);
        }
        event.stop_propagation();
        event.prevent_default();
    }

    fn wheel_amount(&self, event: &WheelEvent) -> f64 {
        let mode = event.delta_mode();
        let y = event.delta_y();
        let browser_mul = match mode {
            0 => 1.,
            1 => 40.,
            _ => 800.
        };
        let config_mul = self.config.wheel_sensitivity;
        y * browser_mul * config_mul
    }

    fn wheel_event(&mut self, event: &WheelEvent) {
        let position = position(&self.lowlevel,event);
        lock!(self.state).pointer.wheel_event(&self.lowlevel,&position,self.wheel_amount(event));
        event.stop_propagation();
        event.prevent_default();
    }
}

pub(super) fn mouse_events(config: &PgPeregrineConfig, state: &LowLevelState, gl: &Arc<Mutex<WebGlGlobal>>, mouse_moved: &Needed) -> Result<Vec<EventHandle>,Message> {
    let mouse_config = Arc::new(PointerConfig::new(config)?);
    let dom = state.dom();
    let canvas = dom.canvas();
    let mut handles = vec![];
    let mut handler = MouseEventHandler::new(mouse_config,state,gl,dom.shutdown());
    confused_browser(canvas.style().set_property("touch-action","none"))?;
    let mut handler2 = handler.clone();
    let mouse_moved2 = mouse_moved.clone();
    handles.push(EventHandle::new(canvas,"pointerdown", move |event: &PointerEvent| {
        handler2.mouse_event(&PointerEventKind::Down,event,&mouse_moved2)
    })?);
    let mut handler2 = handler.clone();
    let mouse_moved2 = mouse_moved.clone();
    handles.push(EventHandle::new(canvas,"pointerup", move |event: &PointerEvent| {
        handler2.mouse_event(&PointerEventKind::Up,event,&mouse_moved2)
    })?);
    let mut handler2 = handler.clone();
    let mouse_moved2 = mouse_moved.clone();
    handles.push(EventHandle::new(canvas,"pointermove", move |event: &PointerEvent| {
        handler2.mouse_event(&PointerEventKind::Move,event,&mouse_moved2)
    })?);
    let mut handler2 = handler.clone();
    handles.push(EventHandle::new(canvas,"wheel", move |event: &WheelEvent| {
        handler2.wheel_event(event)
    })?);
    let mut handler2 = handler.clone();
    handles.push(EventHandle::new(canvas,"pointerleave", move |event: &PointerEvent| {
        handler2.abandon(&event)
    })?);
    handles.push(EventHandle::new(canvas,"scroll", |_: &Event| {})?);
    handles.push(EventHandle::new(canvas,"contextmenu", |_: &Event| {})?);
    Ok(handles)
}
