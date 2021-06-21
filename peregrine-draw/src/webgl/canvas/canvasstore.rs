/* We really don't want to create dom elements when we don't need to, to avoid the dreaded garbage collector.
 * Out flat canvases only come in a few (power of two) sizes and are broadly similar in size. We are fine to waste
 * memory on these, so we keep them and reuse them.
 *
 * We never allocate smaller than MINIMUM in either dimension. After that we only go up by SCALE.
 */

use std::collections::HashMap;
use web_sys::{CanvasRenderingContext2d, Document, HtmlCanvasElement};
use crate::{ util::Message };
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

const MINIMUM : u32 = 256;
const SCALE : u32 = 4;

pub struct HtmlFlatCanvas {
    element: HtmlCanvasElement,
    size: (u32,u32) 
}

impl HtmlFlatCanvas {
    fn new(document: &Document, x: u32, y: u32) -> Result<HtmlFlatCanvas,Message> {
        let element = document.create_element("canvas").map_err(|e| Message::ConfusedWebBrowser(format!("cannot create canvas")))?;
        let element =  element.dyn_into::<HtmlCanvasElement>().map_err(|_| Message::ConfusedWebBrowser("could not cast canvas to HtmlCanvasElement".to_string()))?;
        element.set_width(x);
        element.set_height(y);
        document.body().unwrap().append_child(&element);
        Ok(HtmlFlatCanvas {
            element,
            size: (x,y)
        })
    }

    pub fn element(&self) -> &HtmlCanvasElement { &self.element }
    pub fn size(&self) -> (u32,u32) { self.size }

    pub fn context(&self) -> Result<CanvasRenderingContext2d,Message> {
        self.element
            .get_context("2d").map_err(|_| Message::Canvas2DFailure("cannot get 2d context".to_string()))?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>().map_err(|_| Message::Canvas2DFailure("cannot get 2d context".to_string()))
    }

    fn clear(&self) -> Result<(),Message> {
        let context = self.context()?;
        context.set_fill_style(&JsValue::from_str("rgb(255,255,255)"));
        context.fill_rect(0.,0.,self.size.0 as f64,self.size.1 as f64);
        Ok(())
    }
}

fn rounded(mut v: u32) -> u32 {
    if v < MINIMUM { v = MINIMUM; }
    let mut  power = 1;
    while power < v  {
        power *= SCALE;
    }
    power
}

pub struct CanvasStore {
    canvases: HashMap<(u32,u32),Vec<HtmlFlatCanvas>>
}

impl CanvasStore {
    pub fn new() -> CanvasStore {
        CanvasStore {
            canvases: HashMap::new()
        }
    }

    fn round_up(&self, x: u32, y: u32) -> (u32,u32) {
        (rounded(x),rounded(y))
    }

    pub fn allocate(&mut self, document: &Document, mut x: u32, mut y: u32, round_up: bool) -> Result<HtmlFlatCanvas,Message> {
        if round_up {
            x = rounded(x);
            y = rounded(y);
        }
        if let Some(mut list) = self.canvases.get_mut(&(x,y)) {
            if let Some(value) = list.pop() {
                value.clear()?;
                return Ok(value);
            }
        }
        let mut out = HtmlFlatCanvas::new(document,x,y)?;
        out.clear()?;
        Ok(out)
    }

    pub fn free(&mut self, element: HtmlFlatCanvas) {
        self.canvases.entry((element.size.0,element.size.1)).or_insert_with(|| vec![]).push(element);
    }

    pub fn discard(&mut self) {
        for (_,mut lists) in self.canvases.drain() {
            lists.clear(); // Should destroy nodes
        }
    }
}
