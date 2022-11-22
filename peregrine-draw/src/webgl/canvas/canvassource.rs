/* We really don't want to create dom elements when we don't need to, to avoid the dreaded garbage collector.
 * Out flat canvases only come in a few (power of two) sizes and are broadly similar in size. We are fine to waste
 * memory on these, so we keep them and reuse them.
 *
 * We never allocate smaller than MINIMUM in either dimension. After that we only go up by SCALE.
 */

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use peregrine_toolkit::error::Error;
use peregrine_toolkit::lock;
use peregrine_toolkit::plumbing::lease::Lease;
use peregrine_toolkit::plumbing::lease::LeaseManager;
use web_sys::{Document};
use crate::webgl::CanvasInUse;
use crate::webgl::CanvasWeave;

use super::canvas::Canvas;

const MINIMUM : u32 = 256;
const SCALE : u32 = 2;

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
pub struct CanvasSource {
    canvases: Arc<Mutex<HashMap<(u32,u32),LeaseManager<Canvas,Error>>>>,
    document: Document,
    bitmap_multiplier: f32,
    stats: Stats,
}

impl CanvasSource {
    pub fn new(document: &Document, bitmap_multiplier: f32) -> CanvasSource {
        CanvasSource {
            canvases: Arc::new(Mutex::new(HashMap::new())),
            document: document.clone(),
            bitmap_multiplier,
            stats: Stats::new()
        }
    }

    pub(super) fn allocate(&self, mut x: u32, mut y: u32, round_up: bool) -> Result<Lease<Canvas>,Error> {
        let document = self.document.clone();
        if round_up {
            x = rounded(x);
            y = rounded(y);
        }
        let stats = self.stats.clone();
        let dpr = self.bitmap_multiplier;
        stats.another();
        let mut canvas = lock!(self.canvases).entry((x,y)).or_insert_with(move || {
            LeaseManager::new(move || {
                stats.add(x,y);
                Canvas::new(&document,x,y,dpr)
            })
        }).allocate()?;
        canvas.get_mut().clear()?;
        Ok(canvas)
    }

    pub(crate) fn bitmap_multiplier(&self) -> f32 { self.bitmap_multiplier }

    pub(crate) fn make(&self, weave: &CanvasWeave, size: (u32,u32)) -> Result<CanvasInUse,Error> {
        let lease = self.allocate(size.0, size.1, weave.round_up())?;
        Ok(CanvasInUse::new(lease,weave,size,self.bitmap_multiplier)?)
    }
}
