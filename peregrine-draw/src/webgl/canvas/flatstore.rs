use std::{collections::HashMap, sync::{Arc, Mutex}};
use super::{pngcache::PngCache, weave::CanvasWeave};
use keyed::KeyedOptionalValues;
use peregrine_toolkit::{error::Error, lock };
use web_sys::{ Document };
use super::flat::Flat;
use keyed::keyed_handle;
use super::canvasstore::CanvasStore;

// TODO test discard webgl buffers etc
// TODO document etc to common data structure

keyed_handle!(FlatId);

pub(crate) struct FlatStore {
    bitmap_multiplier: f32,
    canvas_store: CanvasStore,
    scratch: HashMap<CanvasWeave,Flat>,
    main_canvases: KeyedOptionalValues<FlatId,Arc<Mutex<Flat>>>,
    png_cache: PngCache
}

impl FlatStore {
    pub(crate) fn new(bitmap_multiplier: f32) -> FlatStore {
        FlatStore {
            bitmap_multiplier,
            canvas_store: CanvasStore::new(),
            scratch: HashMap::new(),
            main_canvases: KeyedOptionalValues::new(),
            png_cache: PngCache::new(),
        }
    }

    pub(crate) fn bitmap_multiplier(&self) -> f32 { self.bitmap_multiplier }

    pub(crate) fn scratch(&mut self, document: &Document, weave: &CanvasWeave, size: (u32,u32)) -> Result<&mut Flat,Error> {
        let mut use_cached = false;
        if let Some(existing) = self.scratch.get(weave) {
            let ex_size = existing.size();
            if ex_size.0 >= size.0 && ex_size.1 >= size.1 {
                use_cached = true;
            }
        }
        if !use_cached {
            let canvas = Flat::new(&mut self.canvas_store,&self.png_cache,document,&CanvasWeave::Crisp,size,self.bitmap_multiplier)?;
            if let Some(mut old) = self.scratch.insert(weave.clone(),canvas) {
                old.discard();
            }
        }
        Ok(self.scratch.get_mut(weave).unwrap())
    }

    pub(super) fn allocate(&mut self, document: &Document, weave: &CanvasWeave, size: (u32,u32)) -> Result<FlatId,Error> {
        Ok(self.main_canvases.add(Arc::new(Mutex::new(Flat::new(&mut self.canvas_store,&self.png_cache,document,weave,size,self.bitmap_multiplier)?))))
    }

    pub(crate) fn retrieve<F,X>(&self, id: &FlatId, cb: F) -> Result<X,Error>
            where F: FnOnce(&Flat) -> X {
        let x = self.main_canvases.get(&id).map_err(|_| Error::fatal("missing key B"))?;
        let y = lock!(x);
        Ok(cb(&y))
    }

    pub(crate) fn modify<F,X>(&self, id: &FlatId, cb: F) -> Result<X,Error>
            where F: FnOnce(&mut Flat) -> X {
        let x = self.main_canvases.get(&id).map_err(|_| Error::fatal("missing key B"))?;
        let mut y = lock!(x);
        Ok(cb(&mut y))
    }

    pub(crate) fn discard(&mut self, id: &FlatId) -> Result<(),Error> {
        self.modify(id,|flat| flat.discard())??;
        self.main_canvases.remove(id);
        Ok(())
    }

    fn discard_all(&mut self) -> Result<(),Error> {
        for canvas in self.main_canvases.values_mut() {
            lock!(canvas).discard()?;
        }
        self.main_canvases = KeyedOptionalValues::new();
        for (_,mut canvas) in self.scratch.drain() {
            canvas.discard()?;
        }
        Ok(())
    }
}

impl Drop for FlatStore {
    fn drop(&mut self) {
        self.discard_all();
    }
}
