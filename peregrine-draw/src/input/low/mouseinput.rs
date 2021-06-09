use std::sync::Arc;
use crate::{run::PgPeregrineConfig };
use web_sys::{Event, MouseEvent, PointerEvent, WheelEvent};
use crate::util::{ Message };
use crate::util::error::confused_browser;
use super::{event::{ EventSystem }, lowlevel::LowLevelState };
use crate::input::low::pointer::pointer::{ Pointer, PointerEventKind, PointerConfig };

fn position(lowlevel: &LowLevelState, event: &MouseEvent) -> (f64,f64) {
    let rect = lowlevel.dom().canvas_frame().get_bounding_client_rect();
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
            self.downstream = false;
        }
    }

    fn position(&self) -> (f64,f64) { self.position }
    fn downstream(&self) -> bool { self.downstream }
}

pub(super) struct MouseEventHandler {
    pointer: Pointer,
    lowlevel: LowLevelState,
    primary: Finger,
    secondary: Finger,
    config: Arc<PointerConfig>,
}

impl MouseEventHandler {
    fn new(config: Arc<PointerConfig>, lowlevel: &LowLevelState) -> MouseEventHandler {
        MouseEventHandler {
            pointer: Pointer::new(lowlevel,&config),
            lowlevel: lowlevel.clone(),
            primary: Finger::new(),
            secondary: Finger::new(),
            config
        }
    }

    fn report(&mut self, kind: &PointerEventKind) {
        let secondary = if self.primary.downstream() {
            Some(self.secondary.position())
        } else {
            None
        };
        self.pointer.process_event(&self.config,&self.lowlevel,self.primary.position(),secondary,&kind);
    }

    fn abandon(&mut self, event: &PointerEvent) {
        if self.primary.mine(event,&PointerEventKind::Up) {
            self.primary.update_position(&self.lowlevel,event,&PointerEventKind::Up);
            self.report(&PointerEventKind::Up);
        }
        if self.secondary.mine(event,&PointerEventKind::Up) {
            self.secondary.update_position(&self.lowlevel,event,&PointerEventKind::Up);
        }
    }

    fn mouse_event(&mut self, kind: &PointerEventKind, event: &PointerEvent) {
        let mut reported_kind = Some(kind.clone());
        if self.primary.mine(event,kind) {
            self.primary.update_position(&self.lowlevel,event,kind);
        } else if self.secondary.mine(event,kind) {
            self.secondary.update_position(&self.lowlevel,event,kind);
            reported_kind = Some(PointerEventKind::Move);
        } else {
            reported_kind = None;
        }
        if let Some(kind) = reported_kind {
            self.report(&kind);
        }
        event.stop_propagation();
        event.prevent_default();
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
        let position = position(&self.lowlevel,event);
        self.pointer.wheel_event(&self.lowlevel,&position,self.wheel_amount(event));
        event.stop_propagation();
        event.prevent_default();
    }
}

pub(super) fn mouse_events(config: &PgPeregrineConfig, state: &LowLevelState) -> Result<EventSystem<MouseEventHandler>,Message> {
    let mouse_config = Arc::new(PointerConfig::new(config)?);
    let dom = state.dom();
    let canvas = dom.canvas();
    let mut events = EventSystem::new(MouseEventHandler::new(mouse_config,state));
    confused_browser(canvas.style().set_property("touch-action","none"))?;
    events.add(canvas,"pointerdown", |handler,event: &PointerEvent| {
        handler.mouse_event(&PointerEventKind::Down,event)
    })?;
    events.add(canvas,"pointerup", |handler,event: &PointerEvent| {
        handler.mouse_event(&PointerEventKind::Up,event)
    })?;
    events.add(canvas,"pointermove", |handler,event: &PointerEvent| {
        handler.mouse_event(&PointerEventKind::Move,event)
    })?;
    events.add(canvas,"wheel", |handler,event| {
        handler.wheel_event(event)
    })?;
    events.add(canvas,"pointerleave",|handler,event| {
        handler.abandon(&event)
    })?;
    events.add(dom.canvas_frame(),"scroll",|_,_: &Event| {
    })?;
    events.add(dom.canvas_frame(),"contextmenu",|_,e: &Event| {
        e.prevent_default();
    })?;
    Ok(events)
}
