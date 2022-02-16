use wasm_bindgen::{ JsCast };
use js_sys::{ Uint8Array };
use web_sys;
use web_sys::{ AbortController, Request, RequestInit, RequestMode, Response };
use serde_json::Value as JsonValue;
use peregrine_toolkit::url::Url;
use serde_cbor::Value as CborValue;
use crate::integration::timer::Timer;
use crate::util::message::Message;
use super::promise::promise_to_future;

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

fn typed_array_to_vec_u8(v: &Uint8Array) -> Vec<u8> {
    // see https://github.com/rustwasm/wasm-bindgen/pull/1147
    let len = v.length() as usize;
    let mut out = vec![0;len];
    v.copy_to(&mut out);
    out
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

    pub fn set_body_cbor(&mut self, value: &CborValue) -> Result<(),Message> {
        self.set_body(serde_cbor::to_vec(&value).map_err(|e| Message::SerializationError(e.to_string()))?);
        Ok(())
    }

    fn add_abort(&mut self, init: &mut RequestInit, timeout: Option<f64>) -> Result<(),Message> {
        let controller = AbortController::new().map_err(|e| Message::ConfusedWebBrowser(format!("Cannot create abort controller: {:?}",e)))?;
        let signal = controller.signal();
        init.signal(Some(&signal));
        let controller2 = controller.clone();
        if let Some(timeout) = timeout {
            let mut timer = Timer::new(move || controller2.abort());
            timer.go(timeout as i32);
        }
        Ok(())
    }

    async fn get(&mut self) -> Result<Response,Message> {
        let mut init = RequestInit::new();
        init.method(&self.method).mode(RequestMode::Cors);
        if let Some(body) = &self.body {
            let js_body = u8_slice_to_typed_array(body);
            init.body(Some(&js_body));
        }
        self.add_abort(&mut init,self.timeout.clone())?;
        let req = Request::new_with_str_and_init(&self.url,&init).map_err(|e| Message::ConfusedWebBrowser(format!("cannot create request: {:?}",e)))?;
        for (k,v) in &self.headers {
            req.headers().set(k,v).map_err(|e| Message::ConfusedWebBrowser(format!("cannot set header {}={}: {:?}",k,v,e)))?;
        }
        let window = web_sys::window().ok_or_else(|| Message::ConfusedWebBrowser(format!("cannot get window")))?;
        let response = promise_to_future(window.fetch_with_request(&req)).await.map_err(|e| Message::BadBackendConnection(format!("cannot send request: {:?}",e.as_string())))?;
        let response : Response = response.dyn_into().map_err(|e| Message::ConfusedWebBrowser(format!("cannot cast response to response: {:?}",e.as_string())))?;
        if !response.ok() {
            return Err(Message::BadBackendConnection(format!("unexpected status code: {}",response.status())));
        }
        Ok(response)
    }

    pub async fn get_json(&mut self) -> Result<JsonValue,Message> {
        self.add_request_header("Content-Type","application/json");
        let response = self.get().await.map(|r| r.json())?.ok().unwrap();
        let js_json = promise_to_future(response).await.ok().unwrap();
        let json : JsonValue = js_json.into_serde().map_err(|e| Message::SerializationError(e.to_string()))?;
        Ok(json)
    }

    pub async fn get_cbor(&mut self) -> Result<CborValue,Message> {
        self.add_request_header("Content-Type","application/cbor");
        let response = self.get().await.map(|r| r.array_buffer())?.ok().unwrap();
        let array_buffer_value = promise_to_future(response).await.ok().unwrap();
        let buffer: Vec<u8> = typed_array_to_vec_u8(&js_sys::Uint8Array::new(&array_buffer_value));
        let cbor = serde_cbor::from_slice(&buffer).map_err(|e| Message::SerializationError(e.to_string()))?;
        Ok(cbor)
    }
}
