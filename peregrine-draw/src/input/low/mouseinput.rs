use std::sync::Arc;
use crate::{run::PgPeregrineConfig };
use web_sys::{ PointerEvent, Event, WheelEvent };
use crate::util::{ Message };
use crate::util::error::confused_browser;
use super::{event::{ EventSystem }, lowlevel::LowLevelState };
use crate::input::low::pointer::pointer::{ Pointer, PointerEventKind, PointerConfig };

pub(super) struct MouseEventHandler {
    pointer: Pointer,
    lowlevel: LowLevelState,
    position: (f64,f64),
    down_pointer_id: Option<i32>,
    config: Arc<PointerConfig>,
}

impl MouseEventHandler {
    fn new(config: Arc<PointerConfig>, lowlevel: &LowLevelState) -> MouseEventHandler {
        MouseEventHandler {
            pointer: Pointer::new(lowlevel,&config),
            lowlevel: lowlevel.clone(),
            position: (0.,0.),
            down_pointer_id: None,
            config
        }
    }

    fn abandon(&mut self, event: &PointerEvent) {
        let id = event.pointer_id();
        if let Some(down_pointer_id) = self.down_pointer_id {
            if down_pointer_id == id {
                self.pointer.process_event(&self.config,&self.lowlevel,&self.position,&PointerEventKind::Up);
                self.down_pointer_id = None;
            }
        }
    }

    fn mouse_event(&mut self, kind: &PointerEventKind, event: &PointerEvent) {
        let id = event.pointer_id();
        let mut relevant = false;
        match kind {
            PointerEventKind::Down => {
                if self.down_pointer_id.is_none() {
                    self.down_pointer_id = Some(id);
                    relevant = true;
                }
            },
            PointerEventKind::Up => {
                let mut take = false;
                if let Some(down_pointer_id) = self.down_pointer_id {
                    if down_pointer_id == id {
                        take = true;
                        relevant = true;
                    }
                }
                if take {
                    self.down_pointer_id = None;
                }
            },
            PointerEventKind::Move => {
                if let Some(down_pointer_id) = self.down_pointer_id {
                    if down_pointer_id == id {
                        relevant = true;
                    }
                }
            }
        }
        if relevant {
            let rect = self.lowlevel.dom().canvas_frame().get_bounding_client_rect();
            let x = (event.client_x() as f64) - rect.left();
            let y = (event.client_y() as f64) - rect.top();
            self.position = (x,y);
            self.pointer.process_event(&self.config,&self.lowlevel,&self.position,kind);
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
        self.pointer.wheel_event(&self.lowlevel,&self.position,self.wheel_amount(event));
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
