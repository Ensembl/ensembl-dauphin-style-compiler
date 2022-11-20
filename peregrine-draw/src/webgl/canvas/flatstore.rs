use std::{collections::HashMap, hash::Hash, fmt::Debug, sync::{Arc, Mutex}};
use super::{pngcache::PngCache, weave::CanvasWeave};
use peregrine_toolkit::{error::Error, lock, plumbing::lease::Lease };
use web_sys::{ Document };
use super::planecanvas::PlaneCanvasAndContext;
use super::canvasstore::CanvasStore;

// TODO test discard webgl buffers etc
// TODO document etc to common data structure

//keyed_handle!(FlatId);

#[derive(Clone)]
pub struct FlatId(Arc<Mutex<PlaneCanvasAndContext>>);

impl FlatId {
    pub(crate) fn retrieve<F,X>(&self, cb: F) -> X
            where F: FnOnce(&PlaneCanvasAndContext) -> X {
        let y = lock!(self.0);
        cb(&y)
    }

    pub(crate) fn modify<F,X>(&self, cb: F) -> X
            where F: FnOnce(&mut PlaneCanvasAndContext) -> X {
        let mut y = lock!(self.0);
        cb(&mut y)
    }
}

impl PartialEq for FlatId {
    fn eq(&self, other: &Self) -> bool {
        let a = lock!(self.0).id();
        let b = lock!(other.0).id();
        a == b
    }
}

impl Eq for FlatId {}

impl Hash for FlatId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        lock!(self.0).id().hash(state);
    }
}

#[cfg(debug_assertions)]
impl Debug for FlatId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        lock!(self.0).fmt(f)
    }
}

#[derive(Clone)]
pub(crate) struct FlatStore {
    bitmap_multiplier: f32,
    canvas_store: CanvasStore,
    scratch: Arc<Mutex<HashMap<CanvasWeave,PlaneCanvasAndContext>>>,
    png_cache: PngCache
}

impl FlatStore {
    pub(crate) fn new(bitmap_multiplier: f32) -> FlatStore {
        FlatStore {
            bitmap_multiplier,
            canvas_store: CanvasStore::new(),
            scratch: Arc::new(Mutex::new(HashMap::new())),
            png_cache: PngCache::new(),
        }
    }

    pub(crate) fn bitmap_multiplier(&self) -> f32 { self.bitmap_multiplier }

    pub(crate) fn scratch(&mut self, document: &Document, weave: &CanvasWeave, size: (u32,u32)) -> Result<Lease<PlaneCanvasAndContext>,Error> {
        let mut use_cached = false;
        if let Some(existing) = lock!(self.scratch).get(weave) {
            let ex_size = existing.size();
            if ex_size.0 >= size.0 && ex_size.1 >= size.1 {
                use_cached = true;
            }
        }
        if !use_cached {
            let canvas = PlaneCanvasAndContext::new(&mut self.canvas_store,&self.png_cache,document,&CanvasWeave::Crisp,size,self.bitmap_multiplier)?;
            lock!(self.scratch).insert(weave.clone(),canvas);
        }
        let canvas = lock!(self.scratch).remove(weave).unwrap();
        let scratch = self.scratch.clone();
        let weave = weave.clone();
        Ok(Lease::new(move |v| {
            lock!(scratch).insert(weave.clone(),v);
        },canvas))
    }

    pub(super) fn allocate(&mut self, document: &Document, weave: &CanvasWeave, size: (u32,u32)) -> Result<FlatId,Error> {
        Ok(FlatId(Arc::new(Mutex::new(PlaneCanvasAndContext::new(&mut self.canvas_store,&self.png_cache,document,weave,size,self.bitmap_multiplier)?))))
    }
}
