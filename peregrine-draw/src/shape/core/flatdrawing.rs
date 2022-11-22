use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use peregrine_toolkit::error::Error;
use peregrine_toolkit::lock;
use crate::webgl::canvas::tessellate::canvastessellator::{CanvasTessellationPrepare, FlatBoundary};
use crate::webgl::{ CanvasInUse, CanvasAndContext, CanvasWeave, DrawingCanvasesBuilder };
use crate::webgl::global::WebGlGlobal;
use super::texture::{CanvasTextureArea };
use crate::util::message::Message;

pub(crate) trait FlatDrawingItem {
    fn compute_hash(&self) -> Option<u64> { None }
    fn group_hash(&self) -> Option<u64> { None }
    fn calc_size(&self, gl: &mut WebGlGlobal) -> Result<(u32,u32),Error>;
    fn padding(&self, _gl: &mut WebGlGlobal) -> Result<(u32,u32),Error> { Ok((0,0)) }
    fn build(&self, canvas: &mut CanvasAndContext, origin: (u32,u32), size: (u32,u32)) -> Result<(),Error>;
}

#[derive(Clone)]
pub(crate) struct CanvasItemHandle(Arc<Mutex<FlatBoundary>>,Option<CanvasInUse>,Arc<dyn FlatDrawingItem>);

impl CanvasItemHandle {
    pub(crate) fn drawn_area(&self) -> Result<CanvasTextureArea,Message> {
        lock!(self.0).drawn_area()
    }

    pub(crate) fn canvas_id(&self) -> Option<&CanvasInUse> {
        self.1.as_ref()
    }
}

pub(crate) struct FlatDrawingManager {
    weave: CanvasWeave,
    uniform_name: String,
    hashed_items: HashMap<u64,CanvasItemHandle>,
    texts: Vec<CanvasItemHandle>,
    canvas_id: Option<CanvasInUse>
}

impl FlatDrawingManager {
    pub(crate) fn new(weave: &CanvasWeave, uniform_name: &str) -> FlatDrawingManager {
        FlatDrawingManager {
            hashed_items: HashMap::new(),
            texts: vec![],
            canvas_id: None,
            weave: weave.clone(),
            uniform_name: uniform_name.to_string()
        }
    }

    pub(crate) fn add<T>(&mut self, item: T) -> CanvasItemHandle where T: FlatDrawingItem + 'static {
        let hash = item.compute_hash();
        if let Some(hash) = hash {
            if let Some(old) = self.hashed_items.get(&hash) {
                return old.clone();
            }
        }
        let boundary = Arc::new(Mutex::new(FlatBoundary::new()));
        let handle = CanvasItemHandle(boundary.clone(),None,Arc::new(item));
        self.texts.push(handle.clone());
        if let Some(hash) = hash {
            self.hashed_items.insert(hash,handle.clone());
        }
        handle
    }

    fn calc_sizes(&mut self, gl: &mut WebGlGlobal) -> Result<(),Error> {
        self.texts.sort_by_key(|h| h.2.group_hash());
        for item in self.texts.iter_mut() {
            let size = item.2.calc_size(gl)?;
            let padding = item.2.padding(gl)?;
            lock!(item.0).set_size(size,padding);
        }
        Ok(())
    }

    pub(crate) fn draw_on_bitmap(&mut self, gl: &mut WebGlGlobal, drawable: &mut DrawingCanvasesBuilder) -> Result<(),Error> {
        self.calc_sizes(gl)?;
        let mut prepare = CanvasTessellationPrepare::new();
        for boundary in &self.texts {
            prepare.add_size(lock!(boundary.0).size_with_padding()?);
        }
        let (width,height) = self.weave.tessellate(&mut prepare,gl.gpu_spec())?;
        let canvas_id = gl.canvas_source().make(&self.weave,(width,height))?;
        drawable.add_canvas(&canvas_id,&self.uniform_name);
        self.canvas_id = Some(canvas_id.clone());
        let origins = prepare.origin();
        let sizes = prepare.size();
        for (boundary,(origin,size)) in self.texts.iter().zip(origins.iter().zip(sizes.iter())) {
            let mut boundary = lock!(boundary.0);
            boundary.set_origin(*origin);
            boundary.update_padded_size(*size);
        }
        let texts = &mut self.texts; 
        canvas_id.modify(|canvas| {
            for item in texts {
                let boundary = lock!(item.0);
                let size = boundary.size_with_padding()?;
                item.2.build(canvas,boundary.origin()?,size)?;
            }
            Ok(())
        })?;
        Ok(())
    }

    pub(crate) fn canvas_id(&self) ->Option<CanvasInUse> {
        self.canvas_id.as_ref().cloned()
    }
}
