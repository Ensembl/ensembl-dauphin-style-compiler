/*

These calls /manage/ the size of a CANVAS. They do this by /observing/ the size of its CONATINER. They are called in
three circumstances:

1. on each frame to:
   a. do any necessary resizing of the CANVAS.
   b. tell the drawing code how big the CONTAINER currently is.
2. on observing a resize of CONTAINER to:
   a. resize the CANVAS if needed.
   b. set the monostable.
3. on the monostable expiring to:
   a. reset the CANVAS size to a sensible value.

1. On each frame tick() is called. 

1a. The assessment of the correct canvas size is performed by test_update_canvas_size().
If this returns Some(x,y) then the canvas needs to be resized. This is performed by update_canvas_size.

1b. take_pending_drawable_size retrieves any updates to the drawable area noted in paths 2 and 3 and applies the update.

2. On a resize of the container being obeserved check_contaier_size() is called. This updates the container_size
internally and also sets the pending_container_size. This will allow the drawable to be updated on the next tick. If
this method confirms that the container was resized, container_was_resized is called which sets the monostable (2b) and
sets the redraw_needed flag (to ensure that we get a tick).

3. On the monostable expiring, redraw_needed is fired again which will cause a tick. The expiry of themomostable will
be noted in the test_update_canvas_size call.

*/

use std::sync::{ Arc, Mutex };
use crate::util::message::Message;
use web_sys::{HtmlCanvasElement, HtmlElement, WebGlRenderingContext, window };
use super::{dom::PeregrineDom, inner::LockedPeregrineInnerAPI };
use crate::util::resizeobserver::PgResizeObserver;
use crate::PeregrineInnerAPI;
use crate::shape::core::redrawneeded::RedrawNeeded;
use crate::util::monostable::Monostable;

fn screen_size() -> (u32,u32) {
    let window = window().unwrap();
    let screen = window.screen().unwrap();
    (screen.width().ok().unwrap() as u32,screen.height().ok().unwrap() as u32)
}

struct SizeManagerState {
    container_size: Option<(u32,u32)>,
    canvas_element: HtmlElement,
    container_element: HtmlElement,
    resize_observer: Option<PgResizeObserver>,
    pending_container_size: Option<(u32,u32)>,
    booted: bool
}

impl SizeManagerState {
    fn check_container_size(&mut self) -> bool {
        let size = self.container_element.get_bounding_client_rect();
        let (x,y) = (size.width() as u32,size.height() as u32);
        let out = self.container_size.map(|(old_x,old_y)| {
            old_x != x || old_y != y
        }).unwrap_or(true);
        self.container_size = Some((x,y));
        if out {
            self.pending_container_size = self.container_size.clone();
        }
        out
    }

    fn set_observer(&mut self, observer: PgResizeObserver) {
        self.resize_observer = Some(observer);
    }

    fn canvas_size(&self) -> (u32,u32) {
        let size = self.canvas_element.get_bounding_client_rect();
        (size.width() as u32,size.height() as u32)
    }

    fn booted(&self) -> bool { self.booted }
    fn set_booted(&mut self) { self.booted = true; }

    fn test_update_canvas_size(&mut self, active: bool) -> Option<(u32,u32)> {
        let (canvas_x,canvas_y) = self.canvas_size();
        if !self.booted {
            return Some((canvas_x,canvas_y));
        }
        if let Some((container_x,container_y)) = self.container_size {
            if active {
                let (min_x,min_y) = screen_size();
                let min_x = min_x.min(WebGlRenderingContext::MAX_VIEWPORT_DIMS);
                let min_y = min_y.min(WebGlRenderingContext::MAX_VIEWPORT_DIMS);
                if canvas_x < min_x || canvas_y <min_y {
                    return Some((min_x,min_y));
                }
            } else if container_x != canvas_x || container_y != canvas_y {
                return Some((container_x,container_y));
            }
        }
        None
    }

    fn take_pending_drawable_size(&mut self) -> Option<(u32,u32)> {
        self.pending_container_size.take()
    }
}

#[derive(Clone)]
pub(crate) struct SizeManager {
    state: Arc<Mutex<SizeManagerState>>,
    activity_monostable: Monostable,
    redraw_needed: RedrawNeeded,
    canvas_element: HtmlCanvasElement
}

impl SizeManager {
    async fn redraw_needed(web: &mut PeregrineInnerAPI) -> RedrawNeeded {
        web.lock().await.stage.lock().unwrap().redraw_needed().clone()
    }

    pub(crate) async fn new(web: &mut PeregrineInnerAPI, dom: &PeregrineDom) -> Result<SizeManager,Message> {
        let redraw_needed = Self::redraw_needed(web).await;
        let redraw_needed2 = redraw_needed.clone();
        let commander = web.lock().await.commander.clone();
        let container_element = dom.canvas_frame().clone();
        let container_element2 = container_element.clone();
        let canvas_element = dom.canvas().clone();
        let canvas_element2 = canvas_element.clone().into();
        let out = SizeManager {
            state: Arc::new(Mutex::new(SizeManagerState {
                container_size: None,
                resize_observer: None,
                container_element,
                canvas_element: canvas_element2,
                pending_container_size: None,
                booted: false
            })),
            redraw_needed,
            activity_monostable: Monostable::new(&commander,5000., move || {
                redraw_needed2.set();
            }), // XXX configurable
            canvas_element
        };
        let out2 = out.clone();
        let resize_observer = PgResizeObserver::new(web, move|_el| {
            if out2.state.lock().unwrap().check_container_size() {
                out2.container_was_resized();
            }
        })?;
        resize_observer.observe(&container_element2);
        out.state.lock().unwrap().set_observer(resize_observer);
        Ok(out)
    }

    fn container_was_resized(&self) {
        if self.state.lock().unwrap().booted() {
            self.activity_monostable.set();
        }
        self.redraw_needed.set();
    }

    fn update_canvas_size(&self, draw: &mut LockedPeregrineInnerAPI, x: u32, y: u32) -> Result<(),Message> {
        self.canvas_element.set_width(x);
        self.canvas_element.set_height(y);
        *draw.webgl.lock().unwrap().canvas_size() = Some((x,y));
        let mut stage = draw.stage.lock().unwrap();
        //use web_sys::console;
        //console::log_1(&format!("{}x{}",x,y).into());        
        stage.x_mut().set_size(x as f64);
        stage.y_mut().set_size(y as f64);
        Ok(())
    }

    pub(crate) fn tick(&self, draw: &mut LockedPeregrineInnerAPI) -> Result<(),Message> {
        let active = self.activity_monostable.get();
        let resize = self.state.lock().unwrap().test_update_canvas_size(active); // to drop lock immediately
        if let Some((resize_x,resize_y)) = resize {
            self.update_canvas_size(draw,resize_x,resize_y)?;
        }
        let mut state = self.state.lock().unwrap();
        if let Some(drawable) = state.take_pending_drawable_size() {
            let mut stage = draw.stage.lock().unwrap();
            stage.x_mut().set_drawable_size(drawable.0 as f64);
            stage.y_mut().set_drawable_size(drawable.1 as f64);
            draw.target_manager.lock().unwrap().update_size((drawable.0,drawable.1));
            state.set_booted();
        }
        Ok(())
    }
}
