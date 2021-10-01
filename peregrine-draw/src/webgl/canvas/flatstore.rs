use std::collections::HashMap;
use super::{pngcache::PngCache, weave::CanvasWeave};
use keyed::KeyedOptionalValues;
use web_sys::{ Document };
use super::flat::Flat;
use keyed::keyed_handle;
use crate::util::message::Message;
use super::canvasstore::CanvasStore;

// TODO test discard webgl buffers etc
// TODO document etc to common data structure

keyed_handle!(FlatId);

pub(crate) struct FlatStore {
    canvas_store: CanvasStore,
    scratch: HashMap<CanvasWeave,Flat>,
    main_canvases: KeyedOptionalValues<FlatId,Flat>,
    png_cache: PngCache
}

impl FlatStore {
    pub(crate) fn new() -> FlatStore {
        FlatStore {
            canvas_store: CanvasStore::new(),
            scratch: HashMap::new(),
            main_canvases: KeyedOptionalValues::new(),
            png_cache: PngCache::new()
        }
    }

    pub(crate) fn scratch(&mut self, document: &Document, weave: &CanvasWeave, size: (u32,u32)) -> Result<&mut Flat,Message> {
        let mut use_cached = false;
        if let Some(existing) = self.scratch.get(weave) {
            let ex_size = existing.size();
            if ex_size.0 >= size.0 && ex_size.1 >= size.1 {
                use_cached = true;
            }
        }
        if !use_cached {
            let canvas = Flat::new(&mut self.canvas_store,&self.png_cache,document,&CanvasWeave::Crisp,size)?;
            self.scratch.insert(weave.clone(),canvas);
        }
        Ok(self.scratch.get_mut(weave).unwrap())
    }

    pub(super) fn allocate(&mut self, document: &Document, weave: &CanvasWeave, size: (u32,u32)) -> Result<FlatId,Message> {
        Ok(self.main_canvases.add(Flat::new(&mut self.canvas_store,&self.png_cache,document,weave,size)?))
    }

    pub(crate) fn get(&self, id: &FlatId) -> Result<&Flat,Message> {
        self.main_canvases.get(id).map_err(|_| Message::CodeInvariantFailed(format!("missing key B")))
    }

    pub(crate) fn get_mut(&mut self, id: &FlatId) -> Result<&mut Flat,Message> {
        self.main_canvases.get_mut(id).map_err(|_| Message::CodeInvariantFailed(format!("missing key B")))
    }

    pub(crate) fn discard(&mut self, id: &FlatId) -> Result<(),Message> {
        self.main_canvases.get_mut(id).map_err(|_| Message::CodeInvariantFailed(format!("missing key A")))?.discard(&mut self.canvas_store)?;
        self.main_canvases.remove(id);
        Ok(())
    }

    pub(crate) fn discard_all(&mut self) -> Result<(),Message> {
        for canvas in self.main_canvases.values_mut() {
            canvas.discard(&mut self.canvas_store)?;
        }
        self.main_canvases = KeyedOptionalValues::new();
        for (_,mut canvas) in self.scratch.drain() {
            canvas.discard(&mut self.canvas_store)?;
        }
        Ok(())
    }
}

impl Drop for FlatStore {
    fn drop(&mut self) {
        self.discard_all();
    }
}
