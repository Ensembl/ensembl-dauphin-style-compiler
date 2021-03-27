use web_sys::{Document, Element, HtmlCanvasElement, HtmlElement};
use crate::util::message::Message;
use wasm_bindgen::JsCast;
use std::ops::Index;
use js_sys::Math::{ random };
use web_sys::HtmlCollection;

fn to_canvas(e: Element) -> Result<HtmlCanvasElement,Message> {
    e.dyn_into::<web_sys::HtmlCanvasElement>().map_err(|_| Message::BadTemplate(format!("canvas is not a canvas element")))
}

fn get_document(e: &Element) -> Result<Document,Message> {
    e.owner_document().ok_or_else(|| Message::ConfusedWebBrowser(format!("canvas has no document")))
}

fn get_body(e: &Element) -> Result<HtmlElement,Message> {
    get_document(e)?.body().ok_or_else(|| Message::ConfusedWebBrowser(format!("document has no body")))
}

const CHARS : &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-";

fn random_string() -> String {
    let mut out = String::new();
    for _ in 0..16 {
        let index = (random() * (CHARS.len() as f64)).floor() as usize;
        let char = CHARS.index(index..(index+1));
        out.push_str(char);
    }
    out
}

fn unique_element(c: HtmlCollection) -> Result<Element,Message> {
    if c.length() != 1 { return Err(Message::BadTemplate(format!("collection has {} members, expected singleton",c.length()))) }
    c.item(0).ok_or_else(|| Message::BadTemplate(format!("collection has {} members, expected singleton",c.length())))
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

fn setup_dom(el: &Element, html: &str, css: &str) -> Result<Element,Message> {
    let dollar = DollarReplace::new();
    el.set_inner_html(&dollar.replace(html));
    add_css(&el.owner_document().ok_or_else(|| Message::ConfusedWebBrowser(format!("Element has no document")))?,&dollar.replace(css))?;
    Ok(unique_element(el.get_elements_by_class_name(&dollar.replace("$-browser-canvas")))?)
}

pub struct PeregrineDom {
    canvas: HtmlCanvasElement,
    document: Document,
    body: HtmlElement
}

impl PeregrineDom {
    pub fn new(el: &Element, html: &str, css: &str) -> Result<PeregrineDom,Message> {
        let canvas = setup_dom(el,html,css)?;
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
