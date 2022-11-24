use std::{collections::HashMap, sync::{Arc, Mutex}};
use super::{weave::CanvasWeave};
use peregrine_toolkit::{error::Error, lock, plumbing::lease::Lease };
use super::canvasinuse::CanvasAndContext;
use super::canvassource::CanvasSource;

// TODO test discard webgl buffers etc
// TODO document etc to common data structure

#[derive(Clone)]
pub(crate) struct ScratchCanvasAllocator {
    canvas_store: CanvasSource,
    scratch: Arc<Mutex<HashMap<CanvasWeave,CanvasAndContext>>>
}

impl ScratchCanvasAllocator {
    pub(crate) fn new(canvas_source: &CanvasSource) -> ScratchCanvasAllocator {
        ScratchCanvasAllocator {
            canvas_store: canvas_source.clone(),
            scratch: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    pub(crate) fn scratch(&mut self, weave: &CanvasWeave, size: (u32,u32)) -> Result<Lease<CanvasAndContext>,Error> {
        let mut use_cached = false;
        if let Some(existing) = lock!(self.scratch).get(weave) {
            let ex_size = existing.size();
            if ex_size.0 >= size.0 && ex_size.1 >= size.1 {
                use_cached = true;
            }
        }
        if !use_cached {
            let lease = self.canvas_store.allocate(size.0,size.1,false)?;
            let canvas = CanvasAndContext::new(lease,&CanvasWeave::Crisp,self.canvas_store.bitmap_multiplier())?;
            lock!(self.scratch).insert(weave.clone(),canvas);
        }
        let canvas = lock!(self.scratch).remove(weave).unwrap();
        let scratch = self.scratch.clone();
        let weave = weave.clone();
        Ok(Lease::new(move |v| {
            lock!(scratch).insert(weave.clone(),v);
        },canvas))
    }
}
