use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use peregrine_toolkit::error::Error;
use peregrine_toolkit::lock;
use crate::webgl::canvas::tessellate::canvastessellator::{CanvasTessellationPrepare, FlatBoundary};
use crate::webgl::{ CanvasInUse, CanvasAndContext, CanvasWeave, DrawingCanvasesBuilder };
use crate::webgl::global::WebGlGlobal;
use super::texture::{CanvasTextureArea };

pub(crate) trait FlatDrawingItem {
    fn compute_hash(&self) -> Option<u64> { None }
    fn group_hash(&self) -> Option<u64> { None }
    fn calc_size(&self, gl: &mut WebGlGlobal) -> Result<FlatBoundary,Error>;
    fn build(&self, canvas: &mut CanvasAndContext, origin: (u32,u32), size: (u32,u32)) -> Result<(),Error>;
}

#[derive(Clone)]
pub(crate) struct CanvasItemHandle(Arc<Mutex<Option<FlatBoundary>>>,Option<CanvasInUse>,Arc<dyn FlatDrawingItem>);

impl CanvasItemHandle {
    fn boundary<F,X>(&self, cb: F) -> Result<X,Error> where F: FnOnce(&mut FlatBoundary) -> X {
        Ok(cb(lock!(self.0).as_mut().ok_or_else(|| Error::fatal("boundary called on uninititlised item"))?))
    }

    fn apply_item_size(&self, gl: &mut WebGlGlobal) -> Result<(),Error> {
        *lock!(self.0) = Some(self.2.calc_size(gl)?);
        Ok(())
    }

    pub(crate) fn drawn_area(&self) -> Result<CanvasTextureArea,Error> {
        Ok(self.boundary(|x| {
            x.drawn_area()
        })??)
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
        let boundary = Arc::new(Mutex::new(None));
        let handle = CanvasItemHandle(boundary.clone(),None,Arc::new(item));
        self.texts.push(handle.clone());
        if let Some(hash) = hash {
            self.hashed_items.insert(hash,handle.clone());
        }
        handle
    }

    pub(crate) fn draw_on_bitmap(&mut self, gl: &mut WebGlGlobal, drawable: &mut DrawingCanvasesBuilder) -> Result<(),Error> {
        //self.texts.sort_by_key(|h| h.2.group_hash());
        for item in self.texts.iter_mut() {
            item.apply_item_size(gl)?;
        }
        let mut prepare = CanvasTessellationPrepare::new();
        for handle in &self.texts {
            prepare.add(handle.boundary(|x| x.clone())?)?;
        }
        let (width,height) = self.weave.tessellate(&mut prepare,gl.gpu_spec())?;
        let canvas_id = gl.canvas_source().make(&self.weave,(width,height))?;
        drawable.add_canvas(&canvas_id,&self.uniform_name);
        self.canvas_id = Some(canvas_id.clone());
        let texts = &mut self.texts; 
        canvas_id.modify(|canvas| {
            for item in texts {
                let size = item.boundary(|x| x.size_with_padding())??;
                item.2.build(canvas,item.boundary(|x| x.origin())??,size)?;
            }
            Ok(())
        })?;
        Ok(())
    }

    pub(crate) fn canvas_id(&self) ->Option<CanvasInUse> {
        self.canvas_id.as_ref().cloned()
    }
}
