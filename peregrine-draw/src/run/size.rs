use std::sync::{ Arc, Mutex };
use crate::util::message::Message;
use web_sys::{ HtmlElement, HtmlCanvasElement };
use super::dom::PeregrineDom;
use crate::util::resizeobserver::PgResizeObserver;
use crate::PeregrineDraw;
use crate::shape::core::redrawneeded::RedrawNeeded;
use crate::util::monostable::Monostable;

fn round_up(value: f64, sf: u32) -> f64 {
    let x : u64 = value as u64;
    let y = 64-x.leading_zeros()-sf;
    let z = x+(1<<(y-1));
    (((z >> y)+1) << y) as f64
}
struct SizeManagerState {
    container_size: Option<(f64,f64)>,
    canvas_element: HtmlElement,
    container_element: HtmlElement,
    resize_observer: Option<PgResizeObserver>
}

impl SizeManagerState {
    fn check_container_size(&mut self) -> bool {
        let size = self.container_element.get_bounding_client_rect();
        let (x,y) = (size.width(),size.height());
        let out = self.container_size.map(|(old_x,old_y)| {
            old_x != x || old_y != y
        }).unwrap_or(true);
        self.container_size = Some((x,y));
        out
    }

    fn set_observer(&mut self, observer: PgResizeObserver) {
        self.resize_observer = Some(observer);
    }

    fn canvas_size(&self) -> (f64,f64) {
        let size = self.canvas_element.get_bounding_client_rect();
        (size.width(),size.height())
    }

    fn test_update_canvas_size(&self, active: bool) -> Option<(f64,f64)> {
        let (canvas_x,canvas_y) = self.canvas_size();
        if let Some((container_x,container_y)) = self.container_size {
            if active {
                let min_x = round_up(container_x,2); // XXX configurable
                let min_y = round_up(container_y,2); // XXX configurable
                if canvas_x < min_x || canvas_y <min_y {
                    return Some((min_x,min_y));
                }
            } else if container_x != canvas_x || container_y != canvas_y {
                return Some((container_x,container_y));
            }
        }
        None
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
    async fn redraw_needed(web: &mut PeregrineDraw) -> RedrawNeeded {
        web.lock().await.stage.lock().unwrap().redraw_needed().clone()
    }

    pub(crate) async fn new(web: &mut PeregrineDraw, dom: &PeregrineDom) -> Result<SizeManager,Message> {
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
                canvas_element: canvas_element2
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
        self.activity_monostable.set();
        self.redraw_needed.set();
    }

    fn update_canvas_size(&self, x: f64, y: f64) -> Result<(),Message> {
        self.canvas_element.set_width(x as u32);
        self.canvas_element.set_height(y as u32);
        Ok(())
    }

    pub(crate) fn maybe_update_canvas_size(&self) -> Result<(),Message> {
        let active = self.activity_monostable.get();
        let resize = self.state.lock().unwrap().test_update_canvas_size(active); // to drop lock immediately
        if let Some((resize_x,resize_y)) = resize {
            self.update_canvas_size(resize_x,resize_y)?;
        }
        Ok(())
    }
}
