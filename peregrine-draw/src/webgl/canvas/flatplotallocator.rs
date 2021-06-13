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

struct WeaveAllocatorData {
    origin: Vec<(u32,u32)>,
    sizes: Vec<(u32,u32)>
}

struct WeaveAllocator {
    uniform_name: String,
    requests: KeyedData<FlatPlotRequestHandle,Option<WeaveAllocatorData>>,
    weave: CanvasWeave,
    canvas: Option<FlatId>
}

impl WeaveAllocator {
    fn new(weave: &CanvasWeave, uniform_name: &str) -> WeaveAllocator {
        WeaveAllocator {
            uniform_name: uniform_name.to_string(),
            weave: weave.clone(),
            requests: KeyedData::new(),
            canvas: None
        }
    }

    fn add(&mut self, id: FlatPlotRequestHandle, sizes: &[(u32,u32)]) {
        self.requests.insert(&id,WeaveAllocatorData {
            sizes: sizes.to_vec(), origin: vec![]
        });
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

    fn origins(&self, id: &FlatPlotRequestHandle) -> Vec<(u32,u32)> {
        self.requests.get(id).as_ref().unwrap().origin.clone()
    }
}

struct FlatPlotRequest {
    uniform_name: String,
    weave: CanvasWeave,
    sizes: Vec<(u32,u32)>
}

pub(crate) struct FlatPlotAllocator {
    requests: KeyedData<FlatPlotRequestHandle,FlatPlotRequest>
}

impl FlatPlotAllocator {
    pub(crate) fn new() -> FlatPlotAllocator {
        FlatPlotAllocator {
            requests: KeyedData::new()
        }
    }

    pub(crate) fn allocate(&mut self, weave: &CanvasWeave, sizes: &[(u32,u32)], uniform_name: &str) -> FlatPlotRequestHandle {
        self.requests.add(FlatPlotRequest {
            uniform_name: uniform_name.to_string(),
            weave: weave.clone(),
            sizes: sizes.to_vec()
        })
    }

    fn make_weave(&mut self, gl: &mut WebGlGlobal, drawable: &mut DrawingFlatsDrawable, weave: CanvasWeave, uniform_name: &str) -> Result<(),Message> {
        let mut allocator = WeaveAllocator::new(&weave,uniform_name);
        for (id,request) in self.requests.items() {
            if request.weave == weave {
                allocator.add(id,&request.sizes);
            }
        }
        allocator.allocate(gl,drawable)?;
        for (id,request) in self.requests.items() {
            if request.weave == weave {
                let origins = allocator.origins(&id);
                drawable.add(id,allocator.canvas.as_ref().unwrap(),origins,&request.uniform_name);    
            }
        }
        Ok(())
    }

    pub(crate) fn make(mut self, gl: &mut WebGlGlobal) -> Result<DrawingFlatsDrawable,Message> {
        let mut drawable = DrawingFlatsDrawable::new();
        self.make_weave(gl,&mut drawable, CanvasWeave::Crisp,"uSampler")?;
        Ok(drawable)
    }
}
