use std::sync::{Arc, Mutex};
use peregrine_toolkit::{lock};
use peregrine_toolkit_async::sync::needed::Needed;
use web_sys::{HtmlElement, Event};

use crate::{Message, stage::{stage::Stage, axis::ReadStageAxis}, input::low::event::EventHandle};

/* Intersection observer is just used to keep the animation loop alive when the user is fiddling
 * with the scrollbar and doing nothing else. The actual detection and change is all done in
 * the RAF-synchronized update() method.
 */

#[derive(Clone)]
pub(crate) struct YPosDetector {
    old_top: Arc<Mutex<Option<i32>>>,
    el: HtmlElement,
    #[allow(unused)] // keeps event alive
    events: Arc<Vec<EventHandle>>,
}

impl YPosDetector {
    pub(crate) fn new(el: &HtmlElement, redraw_needed: &Needed) -> Result<YPosDetector,Message> {
        let redraw_needed = redraw_needed.clone();
        let events = vec![
            EventHandle::new(el,"scroll",move |_: &Event| {
                redraw_needed.set();
            })?
        ];
        Ok(YPosDetector {
            old_top: Arc::new(Mutex::new(None)),
            events: Arc::new(events),
            el: el.clone()
        })
    }

    fn y_changed(&self, y: i32) {
        let old_value = lock!(self.old_top).clone();
        if let Some(old_value) = old_value {
            if old_value != y {
                self.el.set_scroll_top(y);
            }
        }
    }

    pub(crate) fn add_stage_listener(&self, stage: &mut Stage) {
        let twin = self.clone();
        stage.y_mut().add_listener(move |x| {
            if let Ok(position) = x.position() {
                twin.y_changed(position.round() as i32);
            }
        });
    }

    pub(crate) fn update(&self, stage: &mut Stage) {
        let top = self.el.scroll_top();
        let mut changed = true;
        let mut old_value = lock!(self.old_top);
        if let Some(old_value) = &*old_value {
            if *old_value == top { changed = false; }
        }
        *old_value = Some(top);
        drop(old_value);
        if changed {
            stage.y_mut().set_position(top as f64);
        }
    }
}
