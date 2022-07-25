use commander::cdr_timer;
use peregrine_toolkit::{plumbing::oneshot::OneShot, log_extra};
use peregrine_toolkit_async::sync::retainer::Retainer;
use web_sys::{Document, Element, HtmlCanvasElement, HtmlElement};
use crate::{util::message::Message, PgCommanderWeb};
use wasm_bindgen::JsCast;
use std::ops::Index;
use js_sys::Math::{ random };
use web_sys::HtmlCollection;

include!(concat!(env!("OUT_DIR"), "/env.rs"));

fn to_canvas(e: Element) -> Result<HtmlCanvasElement,Message> {
    e.dyn_into::<web_sys::HtmlCanvasElement>().map_err(|_| Message::BadTemplate(format!("canvas is not a canvas element")))
}

fn to_html(e: Element) -> Result<HtmlElement,Message> {
    e.dyn_into::<web_sys::HtmlElement>().map_err(|_| Message::BadTemplate(format!("element is not an html element!")))
}

fn get_document(e: &Element) -> Result<Document,Message> {
    e.owner_document().ok_or_else(|| Message::ConfusedWebBrowser(format!("canvas has no document")))
}

fn get_body(e: &Element) -> Result<HtmlElement,Message> {
    get_document(e)?.body().ok_or_else(|| Message::ConfusedWebBrowser(format!("document has no body")))
}

fn parent(e: &Element) -> Result<Element,Message> {
    e.parent_element().ok_or_else(|| Message::ConfusedWebBrowser(format!("element has no parent")))
}

const CHARS : &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-";

fn random_string() -> String {
    let mut out = "z".to_string();
    for _ in 0..16 {
        let index = (random() * (CHARS.len() as f64)).floor() as usize;
        let char = CHARS.index(index..(index+1));
        out.push_str(char);
    }
    out
}

fn require<T>(r: Result<Option<T>,Message>) -> Result<T,Message> {
    r.and_then(|v|
        v.ok_or_else(|| Message::BadTemplate(format!("collection has no members, expected singleton")))
    )
}

fn unique_element(c: HtmlCollection) -> Result<Option<Element>,Message> {
    match c.length() {
        0 => Ok(None),
        1 => Ok(c.item(0)),
        _ => return Err(Message::BadTemplate(format!("collection has {} members, expected singleton",c.length())))
    }
}

fn add_css(document: &Document, css: &str) -> Result<(),Message> {
    let style = document.create_element("style").map_err(|e| Message::ConfusedWebBrowser(format!("Cannot create style element: {:?}",e.as_string())))?;
    style.set_text_content(Some(css));
    style.set_attribute("type","text/css").map_err(|e| Message::ConfusedWebBrowser(format!("Cannot set style element attr {:?}",e.as_string())))?;
    let body = document.body().ok_or_else(|| Message::ConfusedWebBrowser(format!("Document has no body")))?;
    body.append_with_node_1(&style).map_err(|e| Message::ConfusedWebBrowser(format!("Cannot append node: {:?}",e.as_string())))?;
    Ok(())
}

struct DollarReplace(String);

impl DollarReplace {
    fn new() -> DollarReplace { DollarReplace(random_string()) }
    fn replace(&self,s: &str) -> String { s.replace("$",&self.0) }
}

fn find_class(dollar: &DollarReplace, el: &Element, class: &str) -> Result<Option<Element>,Message> {
    unique_element(el.get_elements_by_class_name(&dollar.replace(class)))
}

fn setup_dom(dollar: &DollarReplace, el: &Element, html: &str, css: &str) -> Result<(Element,Element),Message> {
    add_css(&el.owner_document().ok_or_else(|| Message::ConfusedWebBrowser(format!("Element has no document")))?,&dollar.replace(css))?;
    el.set_inner_html(&dollar.replace(html));
    let canvas = require(unique_element(el.get_elements_by_class_name(&dollar.replace("$-browser-canvas"))))?;
    let container = require(unique_element(el.get_elements_by_class_name(&dollar.replace("$-container"))))?;
    Ok((canvas,container))
}

#[derive(Clone)]
pub struct PeregrineDom {
    canvas: HtmlCanvasElement,
    canvas_frame: HtmlElement,
    canvas_container: HtmlElement,
    document: Document,
    body: HtmlElement,
    device_pixel_ratio: f32,
    shutdown: OneShot
}

async fn check_for_shutdown(oneshot: &OneShot, element: &HtmlElement) -> Result<bool,Message> {
    if !element.is_connected() {
        log_extra!("shutting down");
        oneshot.run();
        Ok(true)
    } else {
        Ok(false)
    }
}

fn run_shutdown_detector(commander: &PgCommanderWeb, oneshot: &OneShot, element: &HtmlElement) ->Result<(),Message> {
    let oneshot = oneshot.clone();
    let element = element.clone();
    commander.add::<Message>("shutdown detector",20,None,None,Box::pin(async move {
        while !check_for_shutdown(&oneshot,&element).await? {
            cdr_timer(5000.).await;
        }
        Ok(())
    }));
    Ok(())
}

fn effective_dpr() -> f32 {
    let real_dpr = web_sys::window().unwrap().device_pixel_ratio() as f32;
    FORCE_DPR.unwrap_or(real_dpr)
}

impl PeregrineDom {
    pub fn new(el: &Element, html: &str, css: &str) -> Result<PeregrineDom,Message> {
        let dollar = DollarReplace::new();
        let (canvas,container) = setup_dom(&dollar,el,html,css)?;
        let canvas_frame = match find_class(&dollar,&canvas,"$-browser-canvas-frame")? {
            Some(e) => e,
            None => parent(&canvas)?
        };
        let device_pixel_ratio = effective_dpr();
        let shutdown = OneShot::new();
        let canvas_frame = to_html(canvas_frame)?;
        Ok(PeregrineDom {
            document: get_document(&canvas)?,
            body: get_body(&canvas)?,
            canvas: to_canvas(canvas)?,
            canvas_container: to_html(container)?,
            canvas_frame,
            device_pixel_ratio,
            shutdown
        })
    }

    pub(crate) fn run_shutdown_detector(&self, commander:& PgCommanderWeb) {
        run_shutdown_detector(commander, &self.shutdown,&self.canvas_frame);
    }

    pub(crate) fn shutdown(&self) -> &OneShot { &self.shutdown }

    pub(crate) fn canvas(&self) -> &HtmlCanvasElement { &self.canvas }
    pub(crate) fn canvas_frame(&self) -> &HtmlElement { &self.canvas_frame }
    pub(crate) fn document(&self) -> &Document { &self.document }
    pub(crate) fn body(&self) -> &HtmlElement { &self.body }
    pub(crate) fn device_pixel_ratio(&self) -> f32 { self.device_pixel_ratio }

    pub(crate) fn set_useful_height(&self, height: u32) {
        let frame_height = self.canvas_frame.get_bounding_client_rect().height().round() as u32;
        let height = height.max(frame_height);
        self.canvas_container.style().set_property("height",&format!("{}px",height)); // XXX errors
    }

    pub(crate) fn set_canvas_size(&self, width: u32, height: u32) {
        self.canvas().set_width((width as f32*self.device_pixel_ratio) as u32);
        self.canvas().set_height((height as f32*self.device_pixel_ratio) as u32);
        self.canvas().style().set_property("height",&format!("{}px",height));
        self.canvas().style().set_property("width",&format!("{}px",width));
    }
}
