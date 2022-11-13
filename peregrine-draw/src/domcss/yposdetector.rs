use std::sync::{Arc, Mutex};
use peregrine_toolkit::{lock};
use peregrine_toolkit_async::sync::needed::Needed;
use wasm_bindgen::{JsValue};
use web_sys::{HtmlElement};

use crate::{input::low::event::EventSystem, Message, stage::stage::Stage};

/* Intersection observer is just used to keep the animation loop alive when the user is fiddling
 * with the scrollbar and doing nothing else. The actual detection and change is all done in
 * the RAF-synchronized update() method.
 */

#[derive(Clone)]
pub(crate) struct YPosDetector {
    old_top: Arc<Mutex<Option<i32>>>,
    //events: Arc<EventSystem<Needed>>,
    el: HtmlElement
}

impl YPosDetector {
    pub(crate) fn new(el: &HtmlElement, redraw_needed: &Needed) -> Result<YPosDetector,Message> {
        let mut events = EventSystem::new(redraw_needed.clone());
        events.add(el,"scroll",|needed,_ : &JsValue| {
            needed.set();
        })?;
        Ok(YPosDetector {
            old_top: Arc::new(Mutex::new(None)),
            //events: Arc::new(events),
            el: el.clone()
        })
    }

    pub(crate) fn update(&self, stage: &mut Stage) {
        let top = self.el.scroll_top();
        let mut changed = true;
        let mut old_value = lock!(self.old_top);
        if let Some(old_value) = *old_value {
            if old_value == top { changed = false; }
        }
        *old_value = Some(top);
        if changed {
            stage.y_mut().set_position(top as f64);
        }
    }
}
