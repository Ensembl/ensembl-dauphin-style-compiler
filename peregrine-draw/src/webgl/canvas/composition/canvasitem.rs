use std::sync::{Arc, Mutex};
use peregrine_toolkit::{error::Error, lock};
use crate::webgl::{global::WebGlGlobal, canvas::htmlcanvas::canvasinuse::CanvasAndContext };

pub(crate) trait CanvasItem {
    fn compute_hash(&self) -> Option<u64> { None }
    fn group_hash(&self) -> Option<u64> { None }
    fn calc_size(&self, gl: &mut WebGlGlobal) -> Result<CanvasItemSize,Error>;
    fn draw_on_bitmap(&self, canvas: &mut CanvasAndContext, origin: (u32,u32), size: (u32,u32)) -> Result<(),Error>;
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct CanvasItemArea {
    origin: (u32,u32),
    size: (u32,u32)
}

impl CanvasItemArea {
    pub(crate) fn new(origin: (u32,u32), size: (u32,u32)) -> CanvasItemArea {
        CanvasItemArea { origin, size }
    }

    pub(crate) fn origin(&self) -> (u32,u32) { self.origin }
    pub(crate) fn size(&self) -> (u32,u32) { self.size }
}

#[cfg_attr(debug_assertions,derive(Debug))]
pub(crate) struct CanvasItemSize((u32,u32),(u32,u32));

impl CanvasItemSize {
    pub(crate) fn new(size: (u32,u32), padding: (u32,u32)) -> CanvasItemSize {
        CanvasItemSize(size,padding)
    }

    pub(crate) fn extend(&mut self, x: Option<u32>, y: Option<u32>) {
        if let Some(x) = x { self.0.0 = x-self.1.0; }
        if let Some(y) = y { self.0.1 = y-self.1.1; }
    }

    pub(crate) fn unpadded_size(&self) -> (u32,u32) { self.0 }

    pub(crate) fn padded_size(&self) -> (u32,u32) {
        ((self.0).0+(self.1).0,(self.0).1+(self.1).1)
    }

    pub(crate) fn pad_origin(&self, origin: (u32,u32)) -> (u32,u32) {
        (origin.0+(self.1).0/2,origin.1+(self.1).1/2)
    }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct CanvasItemAreaSource(Arc<Mutex<Option<CanvasItemArea>>>);

impl CanvasItemAreaSource {
    pub(crate) fn new() -> CanvasItemAreaSource {
        CanvasItemAreaSource(Arc::new(Mutex::new(None)))
    }

    pub(crate) fn set(&self, area: CanvasItemArea) {
        *lock!(self.0) = Some(area);
    }

    pub(crate) fn get(&self) -> Result<CanvasItemArea,Error> {
        lock!(self.0).as_ref().map(|x| x.clone()).ok_or_else(|| Error::fatal("source not ready"))
    }
}
