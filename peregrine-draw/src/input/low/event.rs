use crate::util::error::{ confused_browser };
use web_sys::HtmlElement;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use crate::Message;

pub(super) fn add_event<T: ?Sized>(element: &HtmlElement, name: &str, closure: &Closure<T>) -> Result<(),Message> {
    confused_browser(element.add_event_listener_with_callback(name,closure.as_ref().unchecked_ref()))
}

pub(super) fn remove_event<T: ?Sized>(element: &HtmlElement, name: &str, closure: &Closure<T>) -> Result<(),Message> {
    confused_browser(element.remove_event_listener_with_callback(name,closure.as_ref().unchecked_ref()))
}
