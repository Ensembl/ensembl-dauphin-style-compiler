use std::sync::{ Arc, Mutex };
use crate::util::message::Message;
use web_sys::{ console, HtmlElement };
use super::dom::PeregrineDom;
use crate::util::resizeobserver::PgResizeObserver;
use crate::PeregrineDraw;

struct SizeManagerState {
    element: HtmlElement,
    resize_observer: Option<PgResizeObserver>
}

impl SizeManagerState {
    pub(super) fn check_size(&self) {
        let size = self.element.get_bounding_client_rect();
        let (x,y) = (size.width(),size.height());
        console::log_1(&format!("entry {},{}",x,y).into());
    }

    fn set_observer(&mut self, observer: PgResizeObserver) {
        self.resize_observer = Some(observer);
    }
}

#[derive(Clone)]
pub(crate) struct SizeManager(Arc<Mutex<SizeManagerState>>);

impl SizeManager {
    pub(crate) fn new(web: &mut PeregrineDraw,dom: &PeregrineDom) -> Result<SizeManager,Message> {
        let element = dom.canvas_frame().clone();
        let element2 = element.clone();
        let out = SizeManager(Arc::new(Mutex::new(SizeManagerState {
            resize_observer: None,
            element
        })));
        let out2 = out.clone();
        let resize_observer = PgResizeObserver::new(web, move|_el| {
            out2.0.lock().unwrap().check_size();
        })?;
        resize_observer.observe(&element2);
        out.0.lock().unwrap().set_observer(resize_observer);
        Ok(out)
    }

    pub(crate) fn xxx(&self) {} // Just so it's not dropped for now
}
