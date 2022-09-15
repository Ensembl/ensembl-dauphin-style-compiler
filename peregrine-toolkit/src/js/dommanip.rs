use std::collections::HashMap;
use wasm_bindgen::JsCast;
use web_sys::{Document, HtmlElement, Element, HtmlCanvasElement, Window, HtmlCollection};

pub fn html_body() -> Result<HtmlElement,String> {
    html_document()?.body().ok_or_else(|| "cannot retrieve body element".to_string())
}

pub fn html_window() -> Result<Window,String> {
    web_sys::window().ok_or_else(|| "cannot retrieve window object".to_string())
}

pub fn html_document() -> Result<Document,String> {
    html_window()?.document().ok_or_else(|| "cannot retrieve document element".to_string())
}

pub fn create_element(name: &str) -> Result<Element,String> {
    html_document()?.create_element(name).map_err(|e| format!("cannot create element {:?}",e))
}

pub fn set_css<T>(el: &HtmlElement, values: &HashMap<&str,T>) -> Result<(),String> where T: AsRef<str> {
    let style = el.style();
    for (key,value) in values {
        style.set_property(key,value.as_ref()).map_err(|e| format!("cannot set css property {:?}",e))?;
    }
    Ok(())
}

pub fn to_canvas(e: HtmlElement) -> Result<HtmlCanvasElement,String> {
    e.dyn_into::<web_sys::HtmlCanvasElement>().map_err(|_| "canvas is not a canvas element".to_string())
}

pub fn to_html(e: Element) -> Result<HtmlElement,String> {
    e.dyn_into::<web_sys::HtmlElement>().ok().ok_or_else(|| "Cannor map element to htmlelement".to_string())
}

pub fn unique_element(c: HtmlCollection) -> Result<Option<Element>,String> {
    match c.length() {
        0 => Ok(None),
        1 => Ok(c.item(0)),
        _ => return Err(format!("collection has {} members, expected singleton",c.length()))
    }
}

pub fn prepend_element(parent: &HtmlElement, child: &HtmlElement) -> Result<(),String> {
    parent.prepend_with_node_1(&child).ok().ok_or_else(|| "Cannot prepend child element".to_string())
}