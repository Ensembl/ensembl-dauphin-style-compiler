use js_sys::Boolean;
use js_sys::Map;
use peregrine_toolkit::error::Error;
use web_sys::{CanvasRenderingContext2d, Document, HtmlCanvasElement};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

pub struct Canvas {
    element: HtmlCanvasElement,
    size: (u32,u32) 
}

impl Canvas {
    pub(super) fn new(document: &Document, x: u32, y: u32) -> Result<Canvas,Error> {
        let element = document.create_element("canvas").map_err(|_| Error::fatal("cannot create canvas"))?;
        let element =  element.dyn_into::<HtmlCanvasElement>().map_err(|_| Error::fatal("could not cast canvas to HtmlCanvasElement"))?;
        element.set_width(x);
        element.set_height(y);
        //document.body().unwrap().append_child(&element);
        Ok(Canvas {
            element,
            size: (x,y)
        })
    }

    pub fn element(&self) -> &HtmlCanvasElement { &self.element }
    pub fn size(&self) -> (u32,u32) { self.size }

    pub fn context(&self) -> Result<CanvasRenderingContext2d,Error> {
        let context_options = Map::new();
        context_options.set(&JsValue::from_str("alpha"),&Boolean::from(JsValue::TRUE));
        context_options.set(&JsValue::from_str("desynchronized"),&Boolean::from(JsValue::TRUE));
        self.element
            .get_context_with_context_options("2d",&context_options)
            .map_err(|_| Error::fatal("cannot get 2d context"))?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>().map_err(|_| Error::fatal("cannot get 2d context"))
    }

    pub(super) fn clear(&self) -> Result<(),Error> {
        let context = self.context()?;
        context.clear_rect(0.,0.,self.size.0 as f64,self.size.1 as f64);
        Ok(())
    }
}
