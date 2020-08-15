use anyhow::{ self, Context };
use wasm_bindgen::{ JsCast, JsValue };
use js_sys::{ Array, Uint8Array };
use wasm_bindgen_futures::JsFuture;
use web_sys::{ Request, RequestInit, RequestMode, Response };
use crate::util::error::{ js_option, js_error, display_error };
use serde_json::Value as JsonValue;

pub struct PgAjax {
    method: String,
    url: String,
    headers: Vec<(String,String)>,
    body: Option<Vec<u8>>
}

fn u8_slice_to_typed_array(v: &[u8]) -> Uint8Array {
    // safe according to https://github.com/rustwasm/wasm-bindgen/issues/1134 and the only way to do it right now
    let len = v.len() as u32;
    unsafe {
        Uint8Array::view(v).slice(0,len)
    }
}

impl PgAjax {
    pub fn new(method: &str, url: &str) -> PgAjax {
        PgAjax {
            method: method.to_string(),
            url: url.to_string(),
            headers: vec![],
            body: None
        }
    }

    pub fn add_request_header(&mut self, key: &str, value: &str) {
        self.headers.push((key.to_string(),value.to_string()))
    }

    pub fn set_body(&mut self, body: Vec<u8>) {
        self.body = Some(body);
    }

    async fn get(&self) -> anyhow::Result<JsValue> {
        let mut init = RequestInit::new();
        init.method(&self.method).mode(RequestMode::Cors);
        if let Some(body) = &self.body {
            let js_body = u8_slice_to_typed_array(body);
            init.body(Some(&js_body));
        }
        let req = js_error(Request::new_with_str_and_init(&self.url,&init))?;
        for (k,v) in &self.headers {
            js_error(req.headers().set(k,v))?;
        }
        let window = js_option(web_sys::window(),"cannot get window")?;
        Ok(js_error(JsFuture::from(window.fetch_with_request(&req)).await)?)
    }

    pub async fn get_json(&mut self) -> anyhow::Result<JsonValue> {
        self.add_request_header("Content-Type","application/json");
        let response = self.get().await?;
        let response: Response = js_error(response.dyn_into()).context("response is not a response!")?;
        let json = js_error(JsFuture::from(js_error(response.json())?).await)?;
        let json : JsonValue = display_error(json.into_serde()).context("not JSON")?;
        Ok(json)
    }
}