use commander::cdr_timer;
use peregrine_toolkit::{plumbing::oneshot::OneShot, log_extra, log};
use web_sys::{Document, Element, HtmlCanvasElement, HtmlElement};
use crate::{util::message::Message, PgCommanderWeb};
use wasm_bindgen::JsCast;
use web_sys::HtmlCollection;

include!(concat!(env!("OUT_DIR"), "/env.rs"));

fn to_canvas(e: HtmlElement) -> Result<HtmlCanvasElement,Message> {
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

fn unique_element(c: HtmlCollection) -> Result<Option<Element>,Message> {
    match c.length() {
        0 => Ok(None),
        1 => Ok(c.item(0)),
        _ => return Err(Message::BadTemplate(format!("collection has {} members, expected singleton",c.length())))
    }
}

fn string_to_dom(document: &Document, html: &str) -> Result<Element,Message> {
    let outer = document.create_element("div").map_err(|e| Message::ConfusedWebBrowser(format!("Cannot create style element: {:?}",e.as_string())))?;
    outer.set_inner_html(html);
    let out = unique_element(outer.children())?;
    out.ok_or_else(|| Message::ConfusedWebBrowser(format!("Cannot find inner element")))
}

#[derive(Clone)]
pub struct PeregrineDom {
    canvas: HtmlCanvasElement,
    canvas_container: HtmlElement,
    viewport_element: HtmlElement,
    content_element: HtmlElement,
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
    pub fn new(el: &Element, html: &str) -> Result<PeregrineDom,Message> {
        let document = el.owner_document().ok_or_else(|| Message::ConfusedWebBrowser(format!("Element has no document")))?;
        let device_pixel_ratio = effective_dpr();
        let shutdown = OneShot::new();
        /* For now, assume the nested document element mode */
        let canvas_container = to_html(el.clone())?;
        let content_element = to_html(unique_element(canvas_container.children())?
            .ok_or_else(|| Message::ConfusedWebBrowser(format!("No inner element")))?)?;
        let canvas = to_html(string_to_dom(&document, html)?)?;
        /* container element must be position: relative so that canvas can be made absolute
         * so that the tops can align and the canvas be on screen. Slightly ugly in API-isolation
         * terms!
         */
        canvas.style().set_property("position", "sticky");
        canvas.style().set_property("display", "block");
        canvas.style().set_property("top", "0");
        canvas.style().set_property("overflow", "hidden");
        content_element.style().set_property("margin-top","0px");
        canvas_container.prepend_with_node_1(&canvas);
        let viewport_element = canvas_container.clone();
        /* */
        Ok(PeregrineDom {
            document: get_document(&canvas)?,
            body: get_body(&canvas)?,
            canvas: to_canvas(canvas)?,
            canvas_container,
            viewport_element,
            content_element,
            device_pixel_ratio,
            shutdown
        })
    }

    pub(crate) fn run_shutdown_detector(&self, commander:& PgCommanderWeb) {
        run_shutdown_detector(commander, &self.shutdown,&self.canvas_container);
    }

    pub(crate) fn shutdown(&self) -> &OneShot { &self.shutdown }

    pub(crate) fn canvas(&self) -> &HtmlCanvasElement { &self.canvas }
    pub(crate) fn viewport_element(&self) -> &HtmlElement { &self.viewport_element }
    pub(crate) fn document(&self) -> &Document { &self.document }
    pub(crate) fn body(&self) -> &HtmlElement { &self.body }
    pub(crate) fn device_pixel_ratio(&self) -> f32 { self.device_pixel_ratio }

    pub(crate) fn set_content_height(&self, height: u32) {
        self.content_element.style().set_property("height",&format!("{}px",height)); // XXX errors
    }

    pub(crate) fn set_canvas_size(&self, width: u32, height: u32) {
        self.canvas().set_width((width as f32*self.device_pixel_ratio) as u32);
        self.canvas().set_height((height as f32*self.device_pixel_ratio) as u32);
        self.canvas().style().set_property("height",&format!("{}px",height));
        self.canvas().style().set_property("width",&format!("{}px",width));
        self.content_element.style().set_property("margin-top",&format!("-{}px",height));
    }
}
