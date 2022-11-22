use std::collections::HashMap;
use peregrine_toolkit::error::Error;
use crate::webgl::canvas::tessellate::canvastessellator::{FlatBoundary, CanvasLocationSource, CanvasItemSize};
use crate::webgl::{ CanvasInUse, CanvasAndContext, CanvasWeave, DrawingCanvasesBuilder };
use crate::webgl::global::WebGlGlobal;

pub(crate) trait FlatDrawingItem {
    fn compute_hash(&self) -> Option<u64> { None }
    fn group_hash(&self) -> Option<u64> { None }
    fn calc_size(&self, gl: &mut WebGlGlobal) -> Result<CanvasItemSize,Error>;
    fn draw_on_bitmap(&self, canvas: &mut CanvasAndContext, origin: (u32,u32), size: (u32,u32)) -> Result<(),Error>;
}

struct CanvasItemHandle {
    source: CanvasLocationSource,
    
    in_progress: Option<FlatBoundary>,
    item: Box<dyn FlatDrawingItem>
}

pub(crate) struct FlatDrawingManager {
    weave: CanvasWeave,
    uniform_name: String,
    hashed_items: HashMap<u64,CanvasLocationSource>,
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

    pub(crate) fn add<T>(&mut self, item: T) -> Result<CanvasLocationSource,Error> where T: FlatDrawingItem + 'static {
        let hash = item.compute_hash();
        if let Some(hash) = hash {
            if let Some(old) = self.hashed_items.get(&hash) {
                return Ok(old.clone());
            }
        }
        let source = CanvasLocationSource::new();
        let handle = CanvasItemHandle {
            source: source.clone(),
            in_progress: None,
            item: Box::new(item)
        };
        self.texts.push(handle);
        if let Some(hash) = hash {
            self.hashed_items.insert(hash,source.clone());
        }
        Ok(source)
    }

    pub(crate) fn draw_on_bitmap(&mut self, gl: &mut WebGlGlobal, drawable: &mut DrawingCanvasesBuilder) -> Result<(),Error> {
        self.texts.sort_by_key(|h| h.item.group_hash());
        for item in self.texts.iter_mut() {
            item.in_progress = Some(FlatBoundary::new(item.item.calc_size(gl)?));
        }
        let mut items = vec![];
        for handle in &mut self.texts {
            items.push(handle.in_progress.as_mut().unwrap());
        }
        let (width,height) = self.weave.tessellate(&mut items,gl.gpu_spec())?;
        let canvas_id = gl.canvas_source().make(&self.weave,(width,height))?;
        drawable.add_canvas(&canvas_id,&self.uniform_name);
        self.canvas_id = Some(canvas_id.clone());
        for handle in &self.texts {
            handle.source.set(handle.in_progress.as_ref().unwrap().area());
        }
        let texts = &mut self.texts;
        canvas_id.modify(|canvas| {
            for item in texts {
                let size = item.in_progress.as_ref().unwrap().size()?;
                let origin = item.in_progress.as_ref().unwrap().origin()?;
                item.item.draw_on_bitmap(canvas,origin,size)?;
            }
            Ok(())
        })?;
        Ok(())
    }

    pub(crate) fn canvas_id(&self) ->Option<CanvasInUse> {
        self.canvas_id.as_ref().cloned()
    }
}
