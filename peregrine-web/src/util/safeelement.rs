use anyhow::{ self, Context, anyhow as err };
use web_sys::{ HtmlElement };
use js_sys::Math::random;
use crate::util::error::{ js_option };
use wasm_bindgen::JsCast;

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

    pub fn get(&self) -> anyhow::Result<HtmlElement> {
        let window = js_option(web_sys::window()).context("getting window")?;
        let document = js_option(window.document()).context("getting document")?;
        let el = js_option(document.get_element_by_id(&self.0)).context("AWOL element")?;
        let html_el = el.dyn_into().or_else(|_| Err(err!("not HTML element")))?;
        Ok(html_el)
    }
}
