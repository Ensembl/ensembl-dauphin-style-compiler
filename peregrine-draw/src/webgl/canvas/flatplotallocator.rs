use keyed::KeyedData;
use std::collections::{ HashMap, HashSet };
use super::packer::allocate_areas;
use crate::webgl::{FlatId, Texture, program::uniform};
use super::weave::{ CanvasWeave };
use super::drawingflats::{ DrawingFlatsDrawable };
use crate::webgl::global::WebGlGlobal;
use keyed::keyed_handle;
use crate::util::message::Message;

keyed_handle!(FlatPlotRequestHandle);

struct FlatPositionAllocatorData {
    origin: Vec<(u32,u32)>,
    sizes: Vec<(u32,u32)>
}

/* Can be multiple per FLAT, orresponds to one entry in DrawingFlatsDrawable */
pub(crate) struct FlatPositionAllocator {
    uniform_name: String,
    requests: KeyedData<FlatPlotRequestHandle,Option<FlatPositionAllocatorData>>,
    weave: CanvasWeave,
    canvas: Option<FlatId>
}

impl FlatPositionAllocator {
    pub(crate) fn new(weave: &CanvasWeave, uniform_name: &str) -> FlatPositionAllocator {
        FlatPositionAllocator {
            uniform_name: uniform_name.to_string(),
            weave: weave.clone(),
            requests: KeyedData::new(),
            canvas: None
        }
    }

    pub(crate) fn insert(&mut self, sizes: &[(u32,u32)]) -> FlatPlotRequestHandle {
        self.requests.add(Some(FlatPositionAllocatorData {
            sizes: sizes.to_vec(), origin: vec![]
        }))
    }

    fn allocate(&mut self, gl: &mut WebGlGlobal, builder: &mut DrawingFlatsDrawable) -> Result<(),Message> {
        let mut sizes = vec![];
        let ids : Vec<_> = self.requests.keys().collect();
        for req_id in &ids {
            let req = self.requests.get(req_id).as_ref().unwrap();
            sizes.extend(req.sizes.iter());
        }
        let (mut origins,width,height) = allocate_areas(&sizes,gl.gpuspec())?;
        let mut origins_iter = origins.drain(..);
        for req_id in &ids {
            let req = self.requests.get_mut(req_id).as_mut().unwrap();
            for _ in 0..req.sizes.len() {
                req.origin.push(origins_iter.next().unwrap());
            }
        }
        self.canvas = Some(builder.make_canvas(gl,&self.weave,(width,height),&self.uniform_name)?);
        Ok(())
    }

    pub(crate) fn origins(&self, id: &FlatPlotRequestHandle) -> Vec<(u32,u32)> {
        self.requests.get(id).as_ref().unwrap().origin.clone()
    }

    pub(crate) fn canvas(&self) -> Result<FlatId,Message> {
        self.canvas.as_ref().cloned().ok_or_else(|| Message::CodeInvariantFailed(format!("no canvas set")))
    }

    pub(crate) fn make(&mut self, gl: &mut WebGlGlobal, drawable: &mut DrawingFlatsDrawable) -> Result<(),Message> {
        self.allocate(gl,drawable)?;
        for (id,_) in self.requests.items() {
            drawable.add(id,self.canvas.as_ref().unwrap());
        }
        Ok(())
    }
}
