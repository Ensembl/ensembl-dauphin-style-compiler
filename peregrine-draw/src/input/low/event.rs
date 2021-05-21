use crate::util::error::{ confused_browser, confused_browser_option };
use web_sys::{ HtmlElement, window, Window };
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use crate::Message;

fn window_catch() -> Result<Window,Message> {
    confused_browser_option(window(),"could not get window object")
}

pub(super) fn add_event<T: ?Sized>(element: &HtmlElement, name: &str, closure: &Closure<T>) -> Result<(),Message> {
    confused_browser(element.add_event_listener_with_callback(name,closure.as_ref().unchecked_ref()))
}

pub(super) fn remove_event<T: ?Sized>(element: &HtmlElement, name: &str, closure: &Closure<T>) -> Result<(),Message> {
    confused_browser(element.remove_event_listener_with_callback(name,closure.as_ref().unchecked_ref()))
}

pub(super) fn window_add_event<T: ?Sized>(name: &str, closure: &Closure<T>) -> Result<(),Message> {
    confused_browser(window_catch()?.add_event_listener_with_callback(name,closure.as_ref().unchecked_ref()))
}

pub(super) fn window_remove_event<T: ?Sized>(name: &str, closure: &Closure<T>) -> Result<(),Message> {
    confused_browser(window_catch()?.remove_event_listener_with_callback(name,closure.as_ref().unchecked_ref()))
}
