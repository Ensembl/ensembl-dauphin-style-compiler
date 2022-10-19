use peregrine_toolkit::error::{Error};
use peregrine_toolkit::{pg_ok, pg_unwrap};
use peregrine_toolkit_async::js::promise::promise_to_future;
use peregrine_toolkit::js::timer::Timer;
use wasm_bindgen::{ JsCast };
use js_sys::{ Uint8Array };
use web_sys;
use web_sys::{ AbortController, Request, RequestInit, RequestMode, Response };
use peregrine_toolkit::url::Url;

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

    fn add_abort(&mut self, init: &mut RequestInit, timeout: Option<f64>) -> Result<(),Error> {
        let controller = pg_ok!(AbortController::new())?;
        let signal = controller.signal();
        init.signal(Some(&signal));
        let controller2 = controller.clone();
        if let Some(timeout) = timeout {
            let mut timer = Timer::new(move || controller2.abort());
            timer.go(timeout as i32);
        }
        Ok(())
    }

    async fn get(&mut self) -> Result<Response,Error> {
        let mut init = RequestInit::new();
        init.method(&self.method).mode(RequestMode::Cors);
        if let Some(body) = &self.body {
            let js_body = u8_slice_to_typed_array(body);
            init.body(Some(&js_body));
        }
        self.add_abort(&mut init,self.timeout.clone())?;
        let req = pg_ok!(Request::new_with_str_and_init(&self.url,&init))?;
        for (k,v) in &self.headers {
            pg_ok!(req.headers().set(k,v))?;
        }
        let window = pg_unwrap!(web_sys::window())?;
        let response = Error::oper_r(
            promise_to_future(window.fetch_with_request(&req)).await,
            "Cannot send request"
        )?;
        let response : Response = pg_ok!(response.dyn_into())?;
        if !response.ok() {
            return Err(Error::operr(&format!("unexpected status code: {}",response.status())));
        }
        Ok(response)
    }

    pub async fn get_cbor(&mut self) -> Result<Vec<u8>,Error> {
        self.add_request_header("Content-Type","application/cbor");
        let response = self.get().await.map(|r| r.array_buffer())?.ok().unwrap();
        let array_buffer_value = promise_to_future(response).await.ok().unwrap();
        Ok(typed_array_to_vec_u8(&js_sys::Uint8Array::new(&array_buffer_value)))
    }
}
