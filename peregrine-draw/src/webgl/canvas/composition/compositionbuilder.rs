use std::collections::HashMap;
use peregrine_toolkit::error::Error;
use peregrine_toolkit::ubail;
use crate::webgl::canvas::binding::weave::CanvasWeave;
use crate::webgl::canvas::composition::canvasitem::{CanvasItemAreaSource};
use crate::webgl::canvas::htmlcanvas::canvasinuse::CanvasInUse;
use crate::webgl::global::WebGlGlobal;
use super::areabuilder::CanvasItemAreaBuilder;
use super::canvasitem::{CanvasItem};

struct ItemInPrgoress {
    source: CanvasItemAreaSource,
    in_progress: Option<CanvasItemAreaBuilder>,
    item: Box<dyn CanvasItem>
}

pub(crate) struct CompositionBuilder {
    weave: CanvasWeave,
    hashed_items: HashMap<u64,CanvasItemAreaSource>,
    texts: Vec<ItemInPrgoress>,
    canvas_id: Option<CanvasInUse>
}

impl CompositionBuilder {
    pub(crate) fn new(weave: &CanvasWeave) -> CompositionBuilder {
        CompositionBuilder {
            hashed_items: HashMap::new(),
            texts: vec![],
            canvas_id: None,
            weave: weave.clone(),
        }
    }

    pub(crate) fn add<T>(&mut self, item: T) -> Result<CanvasItemAreaSource,Error> where T: CanvasItem + 'static {
        let hash = item.compute_hash();
        if let Some(hash) = hash {
            if let Some(old) = self.hashed_items.get(&hash) {
                return Ok(old.clone());
            }
        }
        let source = CanvasItemAreaSource::new();
        let handle = ItemInPrgoress {
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

    pub(crate) fn draw_on_bitmap(&mut self, gl: &mut WebGlGlobal) -> Result<Option<CanvasInUse>,Error> {
        self.texts.sort_by_key(|h| h.item.group_hash());
        for item in self.texts.iter_mut() {
            item.in_progress = Some(CanvasItemAreaBuilder::new(item.item.calc_size(gl)?));
        }
        let mut items = vec![];
        for handle in &mut self.texts {
            items.push(handle.in_progress.as_mut().unwrap());
        }
        let (width,height) = ubail!(self.weave.tessellate(&mut items,gl.gpu_spec())?,Ok(None));
        let canvas_id = gl.canvas_source().make(&self.weave,(width,height))?;
        let (force_width,force_height) = self.weave.force_size(width,height);
        for item in items {
            item.build(force_width,force_height)?;
        }
        self.canvas_id = Some(canvas_id.clone());
        for handle in &self.texts {
            handle.source.set(handle.in_progress.as_ref().unwrap().area());
        }
        let texts = &mut self.texts;
        canvas_id.modify(|canvas| {
            for item in texts {
                let size = item.in_progress.as_ref().unwrap().size();
                let origin = item.in_progress.as_ref().unwrap().origin()?;
                item.item.draw_on_bitmap(canvas,origin,size)?;
            }
            Ok(())
        })?;
        Ok(Some(canvas_id))
    }

    pub(crate) fn canvas(&self) ->Option<CanvasInUse> {
        self.canvas_id.as_ref().cloned()
    }
}
