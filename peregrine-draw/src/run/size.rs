use std::sync::{ Arc, Mutex };
use crate::util::message::Message;
use web_sys::{ HtmlElement };
use super::dom::PeregrineDom;
use crate::util::resizeobserver::PgResizeObserver;
use crate::PeregrineDraw;
use crate::shape::core::redrawneeded::RedrawNeeded;

struct SizeManagerState {
    size: Option<(f64,f64)>,
    element: HtmlElement,
    resize_observer: Option<PgResizeObserver>
}

impl SizeManagerState {
    pub(super) fn check_size(&mut self) -> bool {
        let size = self.element.get_bounding_client_rect();
        let (x,y) = (size.width(),size.height());
        let out = self.size.map(|(old_x,old_y)| {
            old_x != x || old_y != y
        }).unwrap_or(true);
        self.size = Some((x,y));
        out
    }

    fn set_observer(&mut self, observer: PgResizeObserver) {
        self.resize_observer = Some(observer);
    }
}

#[derive(Clone)]
pub(crate) struct SizeManager(Arc<Mutex<SizeManagerState>>,RedrawNeeded);

impl SizeManager {
    async fn redraw_needed(web: &mut PeregrineDraw) -> RedrawNeeded {
        web.lock().await.stage.lock().unwrap().redraw_needed().clone()
    }

    pub(crate) async fn new(web: &mut PeregrineDraw,dom: &PeregrineDom) -> Result<SizeManager,Message> {
        let redraw_needed = Self::redraw_needed(web).await;
        let element = dom.canvas_frame().clone();
        let element2 = element.clone();
        let out = SizeManager(Arc::new(Mutex::new(SizeManagerState {
            size: None,
            resize_observer: None,
            element
        })),redraw_needed);
        let out2 = out.clone();
        let resize_observer = PgResizeObserver::new(web, move|_el| {
            if out2.0.lock().unwrap().check_size() {
                out2.was_resized();
            }
        })?;
        resize_observer.observe(&element2);
        out.0.lock().unwrap().set_observer(resize_observer);
        Ok(out)
    }

    fn was_resized(&self) {
        self.1.set(); // Redraw needed
    }

    pub(crate) fn xxx(&self) {} // Just so it's not dropped for now
}
