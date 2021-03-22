use anyhow::{ self, anyhow as err };
use web_sys::{ HtmlElement };
use js_sys::Math::random;
use crate::util::error::{ js_option };
use wasm_bindgen::JsCast;
use crate::util::message::Message;

#[derive(Clone)]
pub struct SafeElement(String);

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
        let window = js_option(web_sys::window(),"cannot get window")?;
        let document = js_option(window.document(),"cannot get document")?;
        let el = js_option(document.get_element_by_id(&self.0),"Safe element gone AWOL")?;
        let html_el = el.dyn_into().or_else(|_| Err(Message::XXXTmp("not HTML element".to_string())))?;
        Ok(html_el)
    }
}
