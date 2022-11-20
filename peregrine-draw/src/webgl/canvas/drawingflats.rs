use std::collections::HashMap;

use crate::webgl::{CanvasWeave, TextureBindery};
use keyed::KeyedData;
use peregrine_toolkit::error::Error;
use crate::webgl::ProcessBuilder;
use super::flatstore::{ FlatId };
use super::flatplotallocator::FlatPositionCampaignHandle;
use crate::webgl::global::WebGlGlobal;
use crate::util::message::Message;

pub struct DrawingAllFlats {
    main_canvases: HashMap<FlatId,String>
}

impl DrawingAllFlats {
     fn new() -> DrawingAllFlats {
        DrawingAllFlats {
            main_canvases: HashMap::new(),
        }
    }

    fn allocate(&mut self, gl: &mut WebGlGlobal, weave: &CanvasWeave, size: (u32,u32), uniform_name: &str) -> Result<FlatId,Error> {
        let gl_ref = gl.refs();
        let document = gl_ref.document.clone();
        let id = gl_ref.flat_store.allocate(&document,weave,size)?;
        self.main_canvases.insert(id.clone(),uniform_name.to_string());
        Ok(id)
    }

    pub(crate) fn add_process(&self, id: &FlatId, process: &mut ProcessBuilder) -> Result<(),Message> {
        if let Some(uniform_name) = self.main_canvases.get(id) {
            process.set_texture(uniform_name,id)?;
        }
        Ok(())
    }

    pub(crate) fn discard(&mut self, bindery: &mut TextureBindery) -> Result<(),Error> {
        for (id,_) in self.main_canvases.drain() {
            bindery.free(&id)?;
        }
        Ok(())
    }
}

#[cfg(debug_drops)]
impl Drop for DrawingAllFlats {
    fn drop(&mut self) {
        use peregrine_toolkit::log;

        if self.main_canvases.len() > 0 {
            log!("drop DAF wich canvases!");
        }
    }
}

/* One overall, differentiates FLATS */
pub(crate) struct DrawingAllFlatsBuilder {
    responses: KeyedData<FlatPositionCampaignHandle,Option<FlatId>>,
    drawing_flats: DrawingAllFlats
}

impl DrawingAllFlatsBuilder {
    pub(crate) fn new() -> DrawingAllFlatsBuilder {
        DrawingAllFlatsBuilder {
            responses: KeyedData::new(),
            drawing_flats: DrawingAllFlats::new()
        }
    }

    pub(super) fn add(&mut self, id: FlatPositionCampaignHandle, canvas: &FlatId) {
        self.responses.insert(&id,canvas.clone());
    }

    pub(super) fn make_canvas(&mut self, gl: &mut WebGlGlobal, weave: &CanvasWeave, size: (u32,u32), uniform_name: &str) -> Result<FlatId,Error> {
        self.drawing_flats.allocate(gl,weave,size,uniform_name)
    }

    pub(crate) fn built(self) -> DrawingAllFlats { self.drawing_flats }
}
