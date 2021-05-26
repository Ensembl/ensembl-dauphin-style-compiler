use std::sync::Arc;
use crate::{run::PgPeregrineConfig };
use web_sys::{ MouseEvent, Event, WheelEvent };
use crate::util::{ Message };
use super::{event::{ EventSystem }, lowlevel::LowLevelState };
use crate::input::low::pointer::pointer::{ Pointer, PointerEventKind, PointerConfig };

pub(super) struct MouseEventHandler {
    pointer: Pointer,
    lowlevel: LowLevelState,
    position: (f64,f64),
    config: Arc<PointerConfig>,
}

impl MouseEventHandler {
    fn new(config: Arc<PointerConfig>, lowlevel: &LowLevelState) -> MouseEventHandler {
        MouseEventHandler {
            pointer: Pointer::new(lowlevel,&config),
            lowlevel: lowlevel.clone(),
            position: (0.,0.),
            config
        }
    }

    fn abandon(&mut self, _event: &Event) {
        self.pointer.process_event(&self.config,&self.lowlevel,&self.position,&PointerEventKind::Up);
    }

    fn mouse_event(&mut self, kind: &PointerEventKind, event: &MouseEvent) {
        let rect = self.lowlevel.dom().canvas_frame().get_bounding_client_rect();
        let x = (event.client_x() as f64) - rect.left();
        let y = (event.client_y() as f64) - rect.top();
        self.position = (x,y);
        self.pointer.process_event(&self.config,&self.lowlevel,&self.position,kind);
        match kind {
            PointerEventKind::Move => {},
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
        self.pointer.wheel_event(&self.lowlevel,&self.position,self.wheel_amount(event));
        event.stop_propagation();
        event.prevent_default();
    }
}

pub(super) fn mouse_events(config: &PgPeregrineConfig, state: &LowLevelState) -> Result<EventSystem<MouseEventHandler>,Message> {
    let mouse_config = Arc::new(PointerConfig::new(config)?);
    let dom = state.dom();
    let body = dom.body();
    let canvas = dom.canvas();
    let mut events = EventSystem::new(MouseEventHandler::new(mouse_config,state));
    events.add(canvas,"mousedown", |handler,event| {
        handler.mouse_event(&PointerEventKind::Down,event)
    })?;
    events.add(canvas,"mouseup", |handler,event| {
        handler.mouse_event(&PointerEventKind::Up,event)
    })?;
    events.add(body,"mousemove", |handler,event| {
        handler.mouse_event(&PointerEventKind::Move,event)
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
