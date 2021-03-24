use web_sys::{Document, Element, HtmlCanvasElement, HtmlElement};
use crate::util::message::Message;
use wasm_bindgen::JsCast;

fn to_canvas(e: Element) -> Result<HtmlCanvasElement,Message> {
    e.dyn_into::<web_sys::HtmlCanvasElement>().map_err(|_| Message::BadTemplate(format!("canvas is not a canvas element")))
}

fn get_document(e: &Element) -> Result<Document,Message> {
    e.owner_document().ok_or_else(|| Message::ConfusedWebBrowser(format!("canvas has no document")))
}

fn get_body(e: &Element) -> Result<HtmlElement,Message> {
    get_document(e)?.body().ok_or_else(|| Message::ConfusedWebBrowser(format!("document has no body")))
}

pub struct PeregrineDom {
    canvas: HtmlCanvasElement,
    document: Document,
    body: HtmlElement
}

impl PeregrineDom {
    pub fn new(canvas: Element) -> Result<PeregrineDom,Message> {
        Ok(PeregrineDom {
            document: get_document(&canvas)?,
            body: get_body(&canvas)?,
            canvas: to_canvas(canvas)?
        })
    }

    pub(crate) fn canvas(&self) -> &HtmlCanvasElement { &self.canvas }
    pub(crate) fn document(&self) -> &Document { &self.document }
    pub(crate) fn body(&self) -> &HtmlElement { &self.body }
}