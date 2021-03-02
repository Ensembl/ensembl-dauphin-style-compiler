use std::collections::HashMap;
use super::weave::CanvasWeave;
use crate::util::keyed::{ KeyedOptionalValues };
use web_sys::{ Document };
use super::flat::Flat;
use crate::keyed_handle;
use crate::util::error::js_warn;

// TODO test discard webgl buffers etc
// TODO document etc to common data structure

keyed_handle!(FlatId);

pub struct FlatStore {
    scratch: HashMap<CanvasWeave,Flat>,
    main_canvases: KeyedOptionalValues<FlatId,Flat>
}

impl FlatStore {
    pub(crate) fn new() -> FlatStore {
        FlatStore {
            scratch: HashMap::new(),
            main_canvases: KeyedOptionalValues::new()
        }
    }

    pub(crate) fn scratch(&mut self, document: &Document, weave: &CanvasWeave, size: (u32,u32)) -> anyhow::Result<&mut Flat> {
        let mut use_cached = false;
        if let Some(existing) = self.scratch.get(weave) {
            let ex_size = existing.size();
            if ex_size.0 >= size.0 && ex_size.1 >= size.1 {
                use_cached = true;
            }
        }
        if !use_cached {
            let canvas = Flat::new(document,&CanvasWeave::Crisp,size)?;
            self.scratch.insert(weave.clone(),canvas);
        }
        Ok(self.scratch.get_mut(weave).unwrap())
    }

    pub(super) fn allocate(&mut self, document: &Document, weave: &CanvasWeave, size: (u32,u32)) -> anyhow::Result<FlatId> {
        Ok(self.main_canvases.add(Flat::new(document,weave,size)?))
    }

    pub(crate) fn get(&self, id: &FlatId) -> anyhow::Result<&Flat> {
        self.main_canvases.get(id)
    }

    pub(crate) fn discard(&mut self, id: &FlatId) -> anyhow::Result<()> {
        self.main_canvases.get_mut(id)?.discard()?;
        self.main_canvases.remove(id);
        Ok(())
    }

    pub(crate) fn discard_all(&mut self) -> anyhow::Result<()> {
        for canvas in self.main_canvases.values_mut() {
            canvas.discard()?;
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
        js_warn(self.discard_all());
    }
}
