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
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;
use web_sys::HtmlCanvasElement;
use web_sys::{Document};
use crate::webgl::canvas::binding::weave::CanvasWeave;

use super::canvasinuse::CanvasInUse;

const MINIMUM : u32 = 256;

fn rounded(mut v: u32, scale: u32) -> u32 {
    if v < MINIMUM { v = MINIMUM; }
    let mut power = 1;
    while power < v  {
        power *= scale;
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
pub struct CanvasSource {
    canvases: Arc<Mutex<HashMap<(u32,u32),LeaseManager<HtmlCanvasElement,Error>>>>,
    document: Document,
    bitmap_multiplier: f32,
    stats: Stats,
}

pub(super) fn create(document: &Document, x: u32, y: u32) -> Result<HtmlCanvasElement,Error> {
    let element = document.create_element("canvas").map_err(|_| Error::fatal("cannot create canvas"))?;
    let element =  element.dyn_into::<HtmlCanvasElement>().map_err(|_| Error::fatal("could not cast canvas to HtmlCanvasElement"))?;
    element.set_width(x);
    element.set_height(y);
    //document.body().unwrap().append_child(&element);
    Ok(element)
}

fn context(element: &HtmlCanvasElement) -> Result<CanvasRenderingContext2d,Error> {
    let context_options = Map::new();
    context_options.set(&JsValue::from_str("alpha"),&Boolean::from(JsValue::TRUE));
    context_options.set(&JsValue::from_str("desynchronized"),&Boolean::from(JsValue::TRUE));
    element
        .get_context_with_context_options("2d",&context_options)
        .map_err(|_| Error::fatal("cannot get 2d context"))?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>().map_err(|_| Error::fatal("cannot get 2d context"))
}

fn clear(element: &HtmlCanvasElement, size: (u32,u32)) -> Result<(),Error> {
    let context = context(element)?;
    context.clear_rect(0.,0.,size.0 as f64,size.1 as f64);
    Ok(())
}

impl CanvasSource {
    pub fn new(document: &Document, bitmap_multiplier: f32) -> CanvasSource {
        let out = CanvasSource {
            canvases: Arc::new(Mutex::new(HashMap::new())),
            document: document.clone(),
            bitmap_multiplier,
            stats: Stats::new()
        };
        /* ~32MB: small potatoes */
        for x in &[256,1024] {
            for y in &[256,512,1024] {
                for _ in 0..4 {
                    out.allocate(*x,*y,true).ok();
                }
            }
        }
        out
    }

    pub(super) fn allocate(&self, mut x: u32, mut y: u32, round_up: bool) -> Result<(Lease<HtmlCanvasElement>,(u32,u32)),Error> {
        let document = self.document.clone();
        if round_up {
            x = rounded(x,4);
            y = rounded(y,2);
        }
        let stats = self.stats.clone();
        stats.another();
        let canvas = lock!(self.canvases).entry((x,y)).or_insert_with(move || {
            LeaseManager::new(move || {
                stats.add(x,y);
                create(&document,x,y)
            })
        }).allocate()?;
        clear(canvas.get(),(x,y))?;
        Ok((canvas,(x,y)))
    }

    pub(crate) fn bitmap_multiplier(&self) -> f32 { self.bitmap_multiplier }

    pub(crate) fn make(&self, weave: &CanvasWeave, size: (u32,u32)) -> Result<CanvasInUse,Error> {
        let (lease,size) = self.allocate(size.0,size.1,weave.round_up())?;
        Ok(CanvasInUse::new(lease,size,weave,self.bitmap_multiplier)?)
    }
}
