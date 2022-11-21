use std::{collections::HashMap, sync::{Arc, Mutex}};
use super::{pngcache::PngCache, weave::CanvasWeave, canvasinuse::CanvasInUse};
use peregrine_toolkit::{error::Error, lock, plumbing::lease::Lease };
use web_sys::{ Document };
use super::canvasinuse::CanvasAndContext;
use super::canvassource::CanvasSource;

// TODO test discard webgl buffers etc
// TODO document etc to common data structure

#[derive(Clone)]
pub(crate) struct CanvasInUseAllocator {
    bitmap_multiplier: f32,
    canvas_store: CanvasSource,
    scratch: Arc<Mutex<HashMap<CanvasWeave,CanvasAndContext>>>,
    png_cache: PngCache
}

impl CanvasInUseAllocator {
    pub(crate) fn new(document: &Document, bitmap_multiplier: f32) -> CanvasInUseAllocator {
        CanvasInUseAllocator {
            bitmap_multiplier,
            canvas_store: CanvasSource::new(document,bitmap_multiplier),
            scratch: Arc::new(Mutex::new(HashMap::new())),
            png_cache: PngCache::new(),
        }
    }

    pub(crate) fn bitmap_multiplier(&self) -> f32 { self.bitmap_multiplier }

    pub(crate) fn scratch(&mut self, weave: &CanvasWeave, size: (u32,u32)) -> Result<Lease<CanvasAndContext>,Error> {
        let mut use_cached = false;
        if let Some(existing) = lock!(self.scratch).get(weave) {
            let ex_size = existing.size();
            if ex_size.0 >= size.0 && ex_size.1 >= size.1 {
                use_cached = true;
            }
        }
        if !use_cached {
            let canvas = CanvasAndContext::new(&mut self.canvas_store,&self.png_cache,&CanvasWeave::Crisp,size)?;
            lock!(self.scratch).insert(weave.clone(),canvas);
        }
        let canvas = lock!(self.scratch).remove(weave).unwrap();
        let scratch = self.scratch.clone();
        let weave = weave.clone();
        Ok(Lease::new(move |v| {
            lock!(scratch).insert(weave.clone(),v);
        },canvas))
    }

    pub(super) fn allocate(&mut self, document: &Document, weave: &CanvasWeave, size: (u32,u32)) -> Result<CanvasInUse,Error> {
        Ok(CanvasInUse::new(&mut self.canvas_store,&self.png_cache,document,weave,size)?)
    }
}
