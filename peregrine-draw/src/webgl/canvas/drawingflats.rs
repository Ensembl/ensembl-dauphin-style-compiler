use std::collections::HashMap;

use crate::webgl::CanvasWeave;
use keyed::KeyedData;
use crate::webgl::ProcessBuilder;
use super::flatstore::{ FlatId, FlatStore };
use super::flatplotallocator::FlatPlotRequestHandle;
use crate::webgl::global::WebGlGlobal;
use crate::util::message::Message;

pub struct DrawingFlats {
    main_canvases: HashMap<FlatId,String>
}

impl DrawingFlats {
     fn new() -> DrawingFlats {
        DrawingFlats {
            main_canvases: HashMap::new(),
        }
    }

    fn allocate(&mut self, gl: &mut WebGlGlobal, weave: &CanvasWeave, size: (u32,u32), uniform_name: &str) -> Result<FlatId,Message> {
        let document = gl.document().clone();
        let id = gl.canvas_store_mut().allocate(&document,weave,size)?;
        self.main_canvases.insert(id.clone(),uniform_name.to_string());
        Ok(id)
    }

    pub(crate) fn add_process(&self, id: &FlatId, process: &mut ProcessBuilder) -> Result<(),Message> {
        if let Some(uniform_name) = self.main_canvases.get(id) {
            process.set_texture(uniform_name,id)?;
        }
        Ok(())
    }

    pub(crate) fn discard(&mut self, store: &mut FlatStore) -> Result<(),Message> {
        for (id,_) in self.main_canvases.drain() {
            store.discard(&id)?;
        }
        Ok(())
    }
}

/* One overall, differentiates FLATS */
pub(crate) struct DrawingFlatsDrawable {
    responses: KeyedData<FlatPlotRequestHandle,Option<FlatId>>,
    drawing_flats: DrawingFlats
}

impl DrawingFlatsDrawable {
    pub(crate) fn new() -> DrawingFlatsDrawable {
        DrawingFlatsDrawable {
            responses: KeyedData::new(),
            drawing_flats: DrawingFlats::new()
        }
    }

    pub(super) fn add(&mut self, id: FlatPlotRequestHandle, canvas: &FlatId) {
        self.responses.insert(&id,canvas.clone());
    }

    pub(super) fn make_canvas(&mut self, gl: &mut WebGlGlobal, weave: &CanvasWeave, size: (u32,u32), uniform_name: &str) -> Result<FlatId,Message> {
        self.drawing_flats.allocate(gl,weave,size,uniform_name)
    }

    pub(crate) fn built(self) -> DrawingFlats { self.drawing_flats }
}
