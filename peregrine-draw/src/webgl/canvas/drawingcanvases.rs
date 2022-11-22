use std::collections::HashMap;

use crate::webgl::{TextureBindery};
use peregrine_toolkit::error::Error;
use crate::webgl::ProcessBuilder;
use super::canvasinuse::CanvasInUse;
use crate::util::message::Message;

pub struct DrawingCanvases {
    main_canvases: HashMap<CanvasInUse,String>
}

impl DrawingCanvases {
     fn new() -> DrawingCanvases {
        DrawingCanvases {
            main_canvases: HashMap::new(),
        }
    }

    fn allocate(&mut self, canvas: &CanvasInUse, uniform_name: &str) {
        self.main_canvases.insert(canvas.clone(),uniform_name.to_string());
    }

    pub(crate) fn add_process(&self, id: &CanvasInUse, process: &mut ProcessBuilder) -> Result<(),Message> {
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
impl Drop for DrawingCanvases {
    fn drop(&mut self) {
        use peregrine_toolkit::log;

        if self.main_canvases.len() > 0 {
            log!("drop DAF wich canvases!");
        }
    }
}

/* One overall, differentiates FLATS */
pub(crate) struct DrawingCanvasesBuilder {
    responses: Vec<CanvasInUse>,
    drawing_flats: DrawingCanvases
}

impl DrawingCanvasesBuilder {
    pub(crate) fn new() -> DrawingCanvasesBuilder {
        DrawingCanvasesBuilder {
            responses: vec![],
            drawing_flats: DrawingCanvases::new()
        }
    }

    pub(crate) fn add_canvas(&mut self, canvas: &CanvasInUse, uniform_name: &str) {
        self.drawing_flats.allocate(canvas,uniform_name);
        self.responses.push(canvas.clone());
    }

    pub(crate) fn built(self) -> DrawingCanvases { self.drawing_flats }
}
