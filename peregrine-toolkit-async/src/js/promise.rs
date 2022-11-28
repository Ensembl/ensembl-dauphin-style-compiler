use std::sync::{Arc, Mutex};

use commander::PromiseFuture;
use js_sys::Promise;
use peregrine_toolkit::lock;
use wasm_bindgen::{JsValue, prelude::Closure};

struct FetchClosures(Option<Closure<dyn FnMut(JsValue)>>,Option<Closure<dyn FnMut(JsValue)>>);

impl FetchClosures {
    fn clear(&mut self) {
        self.0 = None;
        self.1 = None;
    }
}

pub fn promise_to_cbs<F,G>(promise: Promise, mut success: F, mut failure: G) where F: FnMut(JsValue) + 'static, G: FnMut(JsValue) + 'static {
    let closures = Arc::new(Mutex::new(FetchClosures(None,None)));
    let closures2 = closures.clone();
    let success_cb = move |x| {
        let mut closures2 = lock!(closures2);
        success(x);
        closures2.clear();
    };
    let closures2 = closures.clone();
    let failure_cb = move |x| {
        let mut closures2 = lock!(closures2);
        failure(x);
        closures2.clear();
    };
    let success_cl = Closure::wrap(Box::new(success_cb) as Box<dyn FnMut(_)>);
    let failure_cl = Closure::wrap(Box::new(failure_cb) as Box<dyn FnMut(_)>);
    let mut closures = lock!(closures);
    closures.0 = Some(success_cl);
    closures.1 = Some(failure_cl);
    let promise = promise.then2(&closures.0.as_ref().unwrap(),&closures.1.as_ref().unwrap());
    drop(promise);
}

pub async fn promise_to_future(promise: Promise) -> Result<JsValue,JsValue> {
    let pf = PromiseFuture::new();
    let pf2 = pf.clone();
    let pf3 = pf.clone();
    promise_to_cbs(promise,move |x| { pf2.satisfy(Ok(x)) },move |x| { pf3.satisfy(Err(x)); });
    pf.await
}
