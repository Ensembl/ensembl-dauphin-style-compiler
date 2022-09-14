/*

These calls /manage/ the size of a CANVAS. They do this by /observing/ the size of its 
CONATINER. They are called in three circumstances:

1. on each frame to:
   a. do any necessary resizing of the CANVAS.
   b. tell the drawing code how big the CONTAINER currently is.
   c. tell the core code an approximate bp/px ratio for bumping purposes.
2. on observing a resize of CONTAINER to:
   a. resize the CANVAS if needed.
   b. set the monostable.
3. on the monostable expiring to:
   a. reset the CANVAS size to a sensible value.

1. On each frame tick() is called. 

1a. The assessment of the correct canvas size is performed by test_update_canvas_size().
If this returns Some(x,y) then the canvas needs to be resized. This is performed by 
update_canvas_size.

1b. take_pending_drawable_size retrieves any updates to the drawable area noted in paths
2 and 3 and applies the update.

2. On a resize of the container being obeserved check_contaier_size() is called. This
updates the container_size internally and also sets the pending_container_size. This will
allow the drawable to be updated on the next tick. If this method confirms that the
container was resized, container_was_resized is called which sets the monostable (2b) and
sets the redraw_needed flag (to ensure that we get a tick).

3. On the monostable expiring, redraw_needed is fired again which will cause a tick. The 
expiry of themomostable will be noted in the test_update_canvas_size call.

*/

use std::sync::{ Arc, Mutex };
use crate::run::inner::LockedPeregrineInnerAPI;
use crate::util::{message::Message };
use commander::{cdr_timer};
use peregrine_toolkit::plumbing::oneshot;
use peregrine_toolkit::{log_extra, lock, log};
use peregrine_toolkit_async::sync::needed::Needed;
use web_sys::{ WebGlRenderingContext, window };
use super::{dom::PeregrineDom };
use crate::util::resizeobserver::PgResizeObserver;
use crate::{PeregrineInnerAPI, PgCommanderWeb};
use crate::util::monostable::Monostable;

fn screen_size() -> (u32,u32) {
    let window = window().unwrap();
    let screen = window.screen().unwrap();
    (screen.width().ok().unwrap() as u32,screen.height().ok().unwrap() as u32)
}

struct SizeManagerState {
    container_size: Option<(u32,u32)>,
    dom: PeregrineDom,
    resize_observer: Option<PgResizeObserver>,
    pending_container_size: Option<(u32,u32)>,
    booted: bool
}

impl SizeManagerState {
    fn check_container_size(&mut self) -> bool {
        let x = self.dom.viewport_element().client_width() as u32;
        let y = self.dom.viewport_element().client_height() as u32;
        if x == 0 || y == 0 {
            log_extra!("browser disappeared XXX signal this");
            return false;
        }
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
        let size = self.dom.canvas().get_bounding_client_rect();
        (size.width().round() as u32,size.height().round() as u32)
    }

    fn booted(&self) -> bool { self.booted }
    fn set_booted(&mut self) { self.booted = true; }

    fn test_update_canvas_size(&mut self, active: bool) -> Option<(u32,u32)> {
        let (canvas_x,canvas_y) = self.canvas_size();
        if !self.booted {
            log!("test_update_canvas_size/A({},{})",canvas_x,canvas_y);
            return Some((canvas_x,canvas_y));
        }
        if let Some((container_x,container_y)) = self.container_size {
            if active && false {
                let (min_x,min_y) = screen_size();
                let min_x = min_x.min(WebGlRenderingContext::MAX_VIEWPORT_DIMS);
                let min_y = min_y.min(WebGlRenderingContext::MAX_VIEWPORT_DIMS);
                if canvas_x < min_x || canvas_y <min_y {
                    log!("test_update_canvas_size/B({},{})",min_x,min_y);
                    return Some((min_x,min_y));
                }
            } else if container_x != canvas_x || container_y != canvas_y {
                log!("test_update_canvas_size/C container ({},{}) canvas ({},{})",container_x,container_y,canvas_x,canvas_y);
                return Some((container_x,container_y));
            }
        }
        None
    }

    fn take_pending_drawable_size(&mut self) -> Option<(u32,u32)> {
        self.pending_container_size.take()
    }
}

fn apply_dpr(size: u32, device_pixel_ratio: f32) -> u32 {
    ((size as f32) * device_pixel_ratio) as u32
}

#[derive(Clone)]
pub(crate) struct SizeManager {
    state: Arc<Mutex<SizeManagerState>>,
    activity_monostable: Monostable,
    redraw_needed: Needed,
    dom: PeregrineDom
}

impl SizeManager {
    async fn redraw_needed(web: &mut PeregrineInnerAPI) -> Needed {
        lock!(web.lock().await.stage).redraw_needed().clone()
    }

    pub(crate) async fn new(web: &mut PeregrineInnerAPI, dom: &PeregrineDom) -> Result<SizeManager,Message> {
        let redraw_needed = Self::redraw_needed(web).await;
        let redraw_needed2 = redraw_needed.clone();
        let commander = web.lock().await.commander.clone();
        let out = SizeManager {
            state: Arc::new(Mutex::new(SizeManagerState {
                container_size: None,
                resize_observer: None,
                dom: dom.clone(),
                pending_container_size: None,
                booted: false
            })),
            redraw_needed,
            activity_monostable: Monostable::new(&commander,5000.,&dom.shutdown(), move || {
                redraw_needed2.set();
            }), // XXX configurable
            dom: dom.clone()
        };
        let out2 = out.clone();
        let resize_observer = PgResizeObserver::new(web, move|_el| {
            if lock!(out2.state).check_container_size() {
                out2.container_was_resized();
            }
        }).await?;
        resize_observer.observe(dom.viewport_element());
        lock!(out.state).set_observer(resize_observer);
        Ok(out)
    }

    fn container_was_resized(&self) {
        if lock!(self.state).booted() {
            self.activity_monostable.set();
        }
        self.redraw_needed.set();
    }

    fn update_canvas_size(&self, draw: &mut LockedPeregrineInnerAPI, x: u32, y: u32) -> Result<(),Message> {
        self.dom.set_canvas_size(x,y);
        let device_pixel_ratio = self.dom.device_pixel_ratio();
        *draw.webgl.lock().unwrap().refs().canvas_size = Some((apply_dpr(x,device_pixel_ratio),apply_dpr(y,device_pixel_ratio)));
        let mut stage = lock!(draw.stage);
        stage.x_mut().set_size(x as f64);
        stage.y_mut().set_size(y as f64);
        Ok(())
    }

    pub(crate) fn prepare_for_draw(&self, draw: &mut LockedPeregrineInnerAPI) -> Result<(),Message> {
        let active = self.activity_monostable.get();
        let resize = self.state.lock().unwrap().test_update_canvas_size(active); // to drop lock immediately
        if let Some((resize_x,resize_y)) = resize {
            self.update_canvas_size(draw,resize_x,resize_y)?;
        }
        let mut state = lock!(self.state);
        if let Some(drawable) = state.take_pending_drawable_size() {
            let mut stage = lock!(draw.stage);
            stage.x_mut().set_drawable_size(drawable.0 as f64);
            stage.y_mut().set_drawable_size(drawable.1 as f64);
            drop(stage);
            draw.data_api.set_min_px_per_carriage(drawable.0/2);
            state.set_booted();
        }
        Ok(())
    }

    async fn run_async(&self, api: &PeregrineInnerAPI) -> Result<(),Message> {
        let mut api = api.clone();
        let shutdown = api.lock().await.dom.shutdown().clone();
        let self2 = self.clone();
        loop {
            let mut locked_api = api.lock().await;
            self2.prepare_for_draw(&mut locked_api)?;
            drop(locked_api);
            cdr_timer(1000.).await;
            if shutdown.poll() { break; }
        }
        log_extra!("size loop shutting down");
        Ok(())
    }

    pub(crate) fn run_backup(&self, commander: &PgCommanderWeb, api: &PeregrineInnerAPI) {
        let self2 = self.clone();
        let api = api.clone();
        commander.add::<Message>("size manager",1,None,None, Box::pin(async move {
            self2.run_async(&api).await;
            Ok(())
        }));
    
    }
}
