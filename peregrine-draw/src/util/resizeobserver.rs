use commander::{ cdr_timer, cdr_tick };
use peregrine_data::Commander;
use std::sync::{ Arc, Mutex, Weak };
use wasm_bindgen::prelude::*;
use crate::util::message::Message;
use crate::{ PeregrineInnerAPI, PgCommanderWeb };
use wasm_bindgen::JsCast;
use web_sys::Element;
use js_sys::Array;

// Not implemented yet in web_sys 2021-03-31. We just implement the bare minimum.
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen (extends =::js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type ResizeObserver;

    #[wasm_bindgen(catch, constructor, js_class = "ResizeObserver")]
    pub fn new(callback: &::js_sys::Function) -> Result<ResizeObserver, JsValue>;

    #[wasm_bindgen(method, js_class="ResizeObserver")]
    pub fn observe(this: &ResizeObserver, element: &Element, options: &JsValue);

    #[wasm_bindgen(method, js_class="ResizeObserver")]
    pub fn disconnect(this: &ResizeObserver);

    #[wasm_bindgen (extends = ::js_sys::Object, js_name = ResizeObserverEntry)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type ResizeObserverEntry;    

    #[wasm_bindgen(method,getter)]
    pub fn target(this: &ResizeObserverEntry) -> Element;
}

struct ROPolyfillElement {
    el: Element,
    size: Option<(f64,f64)>
}

impl ROPolyfillElement {
    fn check(&mut self) -> bool {
        let size = self.el.get_bounding_client_rect();
        let (x,y) = (size.width(),size.height());
        let out = self.size.map(|(old_x,old_y)| old_x != x || old_y != y).unwrap_or(true);
        self.size = Some((x,y));
        out
    }
}

async fn ropolyfill_loop(elements: Weak<Mutex<Vec<ROPolyfillElement>>>, cb: Arc<dyn Fn(&Element)>) {
    let productive_ticks = 120;
    let mut productive_timer = 0;
    loop {
        let mut productive = false;
        if let Some(els) = elements.upgrade() {
            for el in els.lock().unwrap().iter_mut() {
                if el.check() {
                    cb(&el.el);
                    productive = true;
                }
            }
            drop(els);
        } else {
            break;
        }
        if productive {
            productive_timer = productive_ticks;
        } else {
            productive_timer = (productive_timer-1).max(0);
        }
        if productive_timer > 0 {
            cdr_tick(1).await;
        } else {
            cdr_timer(500.).await; // XXX configurable
        }
    }
}

#[derive(Clone)]
struct ROPolyfill {
    elements: Arc<Mutex<Vec<ROPolyfillElement>>>
}

impl ROPolyfill {
    fn new(web: &mut PeregrineInnerAPI, cb: Arc<dyn Fn(&Element)>) -> ROPolyfill {
        let elements =  Arc::new(Mutex::new(vec![]));
        let elements2 = elements.clone();
        web.commander().add_task("resize-polyfill",10,None,None,Box::pin(async move {
            ropolyfill_loop(Arc::downgrade(&elements2),cb).await;
            Ok(())
        }));        
        ROPolyfill {
            elements
        }
    }

    fn observe(&self, el: &Element) {
        self.elements.lock().unwrap().push(ROPolyfillElement {
            el: el.clone(),
            size: None
        });
    }
}

enum ROImpl {
    RealImpl(ResizeObserver,Closure<dyn Fn(JsValue,JsValue)>),
    PolyfillImpl(ROPolyfill)
}

pub(crate) struct PgResizeObserver(ROImpl);

impl PgResizeObserver {
    fn try_get_resizeobserver(closure: &Closure<dyn Fn(JsValue,JsValue)>) -> Option<ResizeObserver> {
        ResizeObserver::new(closure.as_ref().unchecked_ref()).ok()
    }

    pub(crate) fn new<F>(web: &mut PeregrineInnerAPI, cb: F) -> Result<PgResizeObserver,Message> where F: Fn(&Element) + 'static {
        let cb = Arc::new(cb);
        let cb2 = cb.clone();
        let closure = Closure::wrap(Box::new(move |entries: JsValue,_observer| {
            let entries = entries.unchecked_into::<Array>();
            for entry in entries.iter() {
                let entry : ResizeObserverEntry = entry.unchecked_ref::<ResizeObserverEntry>().clone();
                let target = entry.target();
                cb(&target);
          }
        }) as Box<dyn Fn(JsValue,JsValue)>);
        Ok(PgResizeObserver(if let Some(resize_observer) = Self::try_get_resizeobserver(&closure) {
            ROImpl::RealImpl(resize_observer,closure)
        } else {
            ROImpl::PolyfillImpl(ROPolyfill::new(web,cb2))
        }))

    }

    pub(crate) fn observe(&self, el: &Element) {
        match &self.0 {
            ROImpl::RealImpl(observer,_) => observer.observe(el,&JsValue::NULL),
            ROImpl::PolyfillImpl(polyfill) => polyfill.observe(el)
        }
    }
}

impl Drop for PgResizeObserver {
    fn drop(&mut self) {
        if let ROImpl::RealImpl(observer,_) = &self.0 {
            observer.disconnect();
        }
    }
}
