/* We really don't want to create dom elements when we don't need to, to avoid the dreaded garbage collector.
 * Out flat canvases only come in a few (power of two) sizes and are broadly similar in size. We are fine to waste
 * memory on these, so we keep them and reuse them.
 *
 * We never allocate smaller than MINIMUM in either dimension. After that we only go up by SCALE.
 */

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use js_sys::Boolean;
use js_sys::Map;
use peregrine_toolkit::error::Error;
use peregrine_toolkit::lock;
use peregrine_toolkit::plumbing::lease::Lease;
use peregrine_toolkit::plumbing::lease::LeaseManager;
use web_sys::{CanvasRenderingContext2d, Document, HtmlCanvasElement};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

const MINIMUM : u32 = 256;
const SCALE : u32 = 2;

pub struct PlaneCanvas {
    element: HtmlCanvasElement,
    size: (u32,u32) 
}

impl PlaneCanvas {
    fn new(document: &Document, x: u32, y: u32) -> Result<PlaneCanvas,Error> {
        let element = document.create_element("canvas").map_err(|_| Error::fatal("cannot create canvas"))?;
        let element =  element.dyn_into::<HtmlCanvasElement>().map_err(|_| Error::fatal("could not cast canvas to HtmlCanvasElement"))?;
        element.set_width(x);
        element.set_height(y);
        //document.body().unwrap().append_child(&element);
        Ok(PlaneCanvas {
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

    fn clear(&self) -> Result<(),Error> {
        let context = self.context()?;
        context.clear_rect(0.,0.,self.size.0 as f64,self.size.1 as f64);
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

#[cfg(debug_canvasstore)]
#[derive(Clone)]
struct Stats(Arc<Mutex<(Vec<(u32,u32)>,usize)>>);

#[cfg(debug_canvasstore)]
impl Stats {
    fn new() -> Stats { Stats(Arc::new(Mutex::new((vec![],0)))) }
    fn another(&self) { lock!(self.0).1 += 1; }
    fn add(&self, x: u32, y: u32) {
        use peregrine_toolkit::log;

        let mut state = lock!(self.0);
        state.0.push((x,y));
        let count : u32 = state.0.iter().map(|c| c.0*c.1/1000).sum();
        log!("{} canvases, {} Mp in play {:?}, {}% cached",
            state.0.len(),count/1000,state,100-state.0.len()*100/state.1.max(1));
    }
}

#[cfg(not(debug_canvasstore))]
#[derive(Clone)]
struct Stats;

#[cfg(not(debug_canvasstore))]
impl Stats {
    fn new() -> Stats { Stats }
    fn another(&self) {}
    fn add(&self, _x: u32, _y: u32) {}
}

#[derive(Clone)]
pub struct CanvasStore {
    canvases: Arc<Mutex<HashMap<(u32,u32),LeaseManager<PlaneCanvas,Error>>>>,
    stats: Stats,
}

impl CanvasStore {
    pub fn new() -> CanvasStore {
        CanvasStore {
            canvases: Arc::new(Mutex::new(HashMap::new())),
            stats: Stats::new()
        }
    }

    pub fn allocate(&self, document: &Document, mut x: u32, mut y: u32, round_up: bool) -> Result<Lease<PlaneCanvas>,Error> {
        let document = document.clone();
        if round_up {
            x = rounded(x);
            y = rounded(y);
        }
        let stats = self.stats.clone();
        stats.another();
        let mut canvas = lock!(self.canvases).entry((x,y)).or_insert_with(move || {
            LeaseManager::new(move || {
                stats.add(x,y);
                PlaneCanvas::new(&document,x,y)
            })
        }).allocate()?;
        canvas.get_mut().clear()?;
        Ok(canvas)
    }
}
