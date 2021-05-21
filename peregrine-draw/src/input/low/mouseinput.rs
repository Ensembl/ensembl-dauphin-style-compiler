use std::collections::{ HashMap, HashSet };
use std::sync::{ Arc, Mutex };
use crate::PeregrineDom;
use wasm_bindgen::prelude::*;
use web_sys::{ MouseEvent, HtmlElement, Event };
use crate::input::{ InputEvent, InputEventKind, Distributor };
use crate::util::{ Message };
use super::event::{ add_event, remove_event, window_add_event, window_remove_event };
use crate::run::PgPeregrineConfig;
use crate::run::{ PgConfigKey };
use super::lowlevel::{ Key, Modifiers };
use js_sys::Date;

pub struct MouseMapBuilder {

}

impl MouseMapBuilder {
    pub fn new() -> MouseMapBuilder {
        MouseMapBuilder {}
    }

    pub fn build(self) -> MouseMap {
        MouseMap {}
    }
}

#[derive(Clone)]
pub struct MouseMap {}

#[derive(Clone,Debug)]
enum MouseEventKind {
    Up,
    Down,
    Move
}

struct MouseEventHandler {
    distributor: Distributor<InputEvent>,
    mapping: MouseMap,
    current: HashSet<InputEventKind>
}

impl MouseEventHandler {
    fn new(distributor: &Distributor<InputEvent>, mapping: &MouseMap) -> MouseEventHandler {
        MouseEventHandler {
            distributor: distributor.clone(),
            mapping: mapping.clone(),
            current: HashSet::new()
        }
    }

    fn abandon(&mut self, event: &Event) {
        use web_sys::console;
        console::log_1(&format!("abandon").into());
    }

    fn mouse_event(&mut self, kind: &MouseEventKind, event: &MouseEvent) {
        use web_sys::console;
        console::log_1(&format!("{:?}",kind).into());
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
    pub fn new(distributor: &Distributor<InputEvent>, dom: &PeregrineDom, mapping: &MouseMap) -> Result<MouseInput,Message> {
        let body = dom.body();
        let handler = Arc::new(Mutex::new(MouseEventHandler::new(distributor,mapping)));
        let up_closure = make_mouse_event(MouseEventKind::Up,&handler);
        let down_closure = make_mouse_event(MouseEventKind::Down,&handler);
        let move_closure = make_mouse_event(MouseEventKind::Move,&handler);
        let blur_closure = make_event(&handler);
        add_event(body,"mousedown",&down_closure)?;
        add_event(body,"mouseup",&up_closure)?;
        add_event(body,"mousemove",&move_closure)?;
        window_add_event("blur",&blur_closure)?;
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
        window_remove_event("blur",&self.blur_closure.as_ref())?;
        Ok(())
    }
}

impl Drop for MouseInput {
    fn drop(&mut self) {
        self.finish().ok();
    }
}
