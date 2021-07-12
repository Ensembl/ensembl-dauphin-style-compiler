use anyhow::{ self, anyhow as err };
use web_sys::{ HtmlElement };
use js_sys::Math::random;
use wasm_bindgen::JsCast;
use crate::util::message::Message;

#[derive(Clone)]
pub struct SafeElement(String);

// TODO check FlatSotre discard

impl SafeElement {
    pub fn new(el: &HtmlElement) -> SafeElement {
        let mut id = el.id();
        if id == "" {
            id = format!("safeelement-{}",(random()*100000000.).floor());
            el.set_id(&id);
        }
        SafeElement(id)
    }

    pub fn get(&self) -> Result<HtmlElement,Message> {
        let window = web_sys::window().ok_or_else(|| Message::ConfusedWebBrowser(format!("cannot get window")))?;
        let document = window.document().ok_or_else(|| Message::ConfusedWebBrowser(format!("cannot get window")))?;
        let el = document.get_element_by_id(&self.0).ok_or_else(|| Message::ConfusedWebBrowser(format!("Safe element gone AWOL")))?;
        let html_el = el.dyn_into().or_else(|_| Err(Message::ConfusedWebBrowser("not HTML element".to_string())))?;
        Ok(html_el)
    }
}
