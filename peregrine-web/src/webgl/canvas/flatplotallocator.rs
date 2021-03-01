use crate::util::keyed::KeyedData;
use std::collections::{ HashMap, HashSet };
use super::packer::allocate_areas;
use super::flatstore::FlatId;
use super::weave::{ CanvasWeave };
use super::drawingflats::{ DrawingFlatsDrawable };
use crate::webgl::global::WebGlGlobal;
use crate::keyed_handle;

keyed_handle!(FlatPlotRequestHandle);

struct WeaveAllocatorData {
    origin: Vec<(u32,u32)>,
    sizes: Vec<(u32,u32)>
}

struct WeaveAllocator {
    requests: KeyedData<FlatPlotRequestHandle,Option<WeaveAllocatorData>>,
    weave: CanvasWeave,
    canvas: Option<FlatId>
}

impl WeaveAllocator {
    fn new(weave: &CanvasWeave) -> WeaveAllocator {
        WeaveAllocator {
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

    fn allocate(&mut self, gl: &mut WebGlGlobal, builder: &mut DrawingFlatsDrawable) -> anyhow::Result<()> {
        let mut sizes = vec![];
        let ids : Vec<_> = self.requests.keys().collect();
        for req_id in &ids {
            let req = self.requests.get(req_id).as_ref().unwrap();
            sizes.extend(req.sizes.iter());
        }
        let (mut origins,width,height) = allocate_areas(&sizes,gl.program_store().gpu_spec())?;
        let mut origins_iter = origins.drain(..);
        for req_id in &ids {
            let req = self.requests.get_mut(req_id).as_mut().unwrap();
            for _ in 0..req.sizes.len() {
                req.origin.push(origins_iter.next().unwrap());
            }
        }
        self.canvas = Some(builder.make_canvas(gl,&self.weave,(width,height))?);
        Ok(())
    }

    fn origins(&self, id: &FlatPlotRequestHandle) -> Vec<(u32,u32)> {
        self.requests.get(id).as_ref().unwrap().origin.clone()
    }
}

struct FlatPlotRequest {
    weave: CanvasWeave,
    sizes: Vec<(u32,u32)>
}

pub struct FlatPlotAllocator {
    requests: KeyedData<FlatPlotRequestHandle,FlatPlotRequest>
}

impl FlatPlotAllocator {
    pub(crate) fn new() -> FlatPlotAllocator {
        FlatPlotAllocator {
            requests: KeyedData::new()
        }
    }

    pub(crate) fn allocate(&mut self, weave: &CanvasWeave, sizes: &[(u32,u32)]) -> FlatPlotRequestHandle {
        self.requests.add(FlatPlotRequest {
            weave: weave.clone(),
            sizes: sizes.to_vec()
        })
    }

    fn all_weaves(&self) -> Vec<CanvasWeave> {
        let mut out = HashSet::new();
        for request in self.requests.values() {
            out.insert(request.weave.clone());
        }
        out.iter().cloned().collect()
    }

    pub(crate) fn make(self, gl: &mut WebGlGlobal,) -> anyhow::Result<DrawingFlatsDrawable> {
        let mut weave_allocators = HashMap::new();
        let all_weaves = self.all_weaves();
        for weave in all_weaves.iter() {
            weave_allocators.insert(weave,WeaveAllocator::new(weave));
        } 
        for (id,request) in self.requests.items() {
            weave_allocators.get_mut(&request.weave).unwrap().add(id,&request.sizes);
        }
        let mut drawable = DrawingFlatsDrawable::new(gl.program_store().gpu_spec());
        for weave_allocator in weave_allocators.values_mut() {
            weave_allocator.allocate(gl,&mut drawable)?;
        }
        for (id,request) in self.requests.items() {
            let weave_allocator = weave_allocators.get(&request.weave).unwrap();
            let origins = weave_allocator.origins(&id);
            drawable.add(id,weave_allocator.canvas.as_ref().unwrap(),origins);
        }
        Ok(drawable)
    }
}
