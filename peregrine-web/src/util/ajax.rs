use std::future::Future;
use anyhow::{ self, Context };
use wasm_bindgen::{ JsCast, JsValue };
use js_sys::{ Array, Uint8Array };
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys;
use web_sys::{ AbortController, Request, RequestInit, RequestMode, Response };
use crate::util::error::{ js_option, js_error, display_error };
use serde_json::Value as JsonValue;
use url::Url;
use serde_cbor::Value as CborValue;

pub struct PgAjax {
    method: String,
    url: String,
    headers: Vec<(String,String)>,
    timeout: Option<f64>,
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
    pub fn new(method: &str, url: &Url) -> PgAjax {
        PgAjax {
            method: method.to_string(),
            url: url.to_string(),
            headers: vec![],
            body: None,
            timeout: None
        }
    }

    pub fn add_request_header(&mut self, key: &str, value: &str) {
        self.headers.push((key.to_string(),value.to_string()))
    }

    pub fn set_timeout(&mut self, timeout: f64) {
        self.timeout = Some(timeout);
    }

    pub fn set_body(&mut self, body: Vec<u8>) {
        self.body = Some(body);
    }

    pub fn set_body_cbor(&mut self, value: &CborValue) -> anyhow::Result<()> {
        self.set_body(serde_cbor::to_vec(&value).context("building trace bytes")?);
        Ok(())
    }

    fn add_timeout(&self, init: &mut RequestInit, timeout: f64) -> anyhow::Result<()> {
        let controller = js_error(AbortController::new())?;
        let signal = controller.signal();
        init.signal(Some(&signal));
        let closure = Closure::once_into_js(Box::new(move || controller.abort()) as Box<dyn Fn()>);
        let window = display_error(web_sys::window().ok_or("cannot get window object"))?;
        js_error(window.set_timeout_with_callback_and_timeout_and_arguments_0(&closure.into(),timeout as i32))?;
        Ok(())
    }

    async fn get(&self) -> anyhow::Result<JsValue> {
        let mut init = RequestInit::new();
        init.method(&self.method).mode(RequestMode::Cors);
        if let Some(body) = &self.body {
            let js_body = u8_slice_to_typed_array(body);
            init.body(Some(&js_body));
        }
        if let Some(timeout) = self.timeout {
            self.add_timeout(&mut init,timeout)?;
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

    pub async fn get_cbor(&mut self) -> anyhow::Result<CborValue> {
        self.add_request_header("Content-Type","application/cbor");
        let response = self.get().await?;
        let response: Response = js_error(response.dyn_into()).context("response is not a response!")?;
        let cbor = js_error(JsFuture::from(js_error(response.text())?).await)?;
        let cbor : CborValue = display_error(cbor.into_serde()).context("not CBOR")?;
        Ok(cbor)
    }
}