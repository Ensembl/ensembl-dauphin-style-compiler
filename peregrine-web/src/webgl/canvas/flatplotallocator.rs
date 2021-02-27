use crate::util::keyed::KeyedData;
use std::collections::{ HashMap, HashSet };
use super::packer::allocate_areas;
use super::flatstore::{ FlatStore, FlatId };
use super::weave::{ CanvasWeave };
use super::drawingflats::{ DrawingFlatsDrawable };
use crate::webgl::GPUSpec;
use crate::keyed_handle;

keyed_handle!(FlatPlotRequestHandle);

struct PerWeaveFlatPlotAllocatorData {
    origin: Vec<(u32,u32)>,
    sizes: Vec<(u32,u32)>
}

struct PerWeaveFlatPlotAllocator {
    requests: HashMap<FlatPlotRequestHandle,PerWeaveFlatPlotAllocatorData>,
    weave: CanvasWeave,
    canvas: Option<FlatId>
}

impl PerWeaveFlatPlotAllocator {
    fn new(weave: &CanvasWeave) -> PerWeaveFlatPlotAllocator {
        PerWeaveFlatPlotAllocator {
            weave: weave.clone(),
            requests: HashMap::new(),
            canvas: None
        }
    }

    fn add(&mut self, id: FlatPlotRequestHandle, sizes: &[(u32,u32)]) {
        self.requests.insert(id.clone(),PerWeaveFlatPlotAllocatorData {
            sizes: sizes.to_vec(), origin: vec![]
        });
    }

    fn allocate(&mut self, store: &mut FlatStore, builder: &mut DrawingFlatsDrawable, gpuspec: &GPUSpec) -> anyhow::Result<()> {
        let mut sizes = vec![];
        let ids : Vec<_> = self.requests.keys().cloned().collect();
        for req_id in &ids {
            let req = self.requests.get(req_id).unwrap();
            sizes.extend(req.sizes.iter());
        }
        let (mut origins,width,height) = allocate_areas(&sizes,gpuspec)?;
        let mut origins_iter = origins.drain(..);
        for req_id in &ids {
            let req = self.requests.get_mut(req_id).unwrap();
            for _ in 0..req.sizes.len() {
                req.origin.push(origins_iter.next().unwrap());
            }
        }
        self.canvas = Some(builder.make_canvas(store,&self.weave,(width,height))?);
        Ok(())
    }

    fn origins(&self, id: &FlatPlotRequestHandle) -> Vec<(u32,u32)> {
        self.requests.get(id).unwrap().origin.clone()
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

    pub(crate) fn allocate_areas(&mut self, weave: &CanvasWeave, sizes: &[(u32,u32)]) -> FlatPlotRequestHandle {
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

    pub(crate) fn make_builder(self, canvas_store: &mut FlatStore, gpu_spec: &GPUSpec) -> anyhow::Result<DrawingFlatsDrawable> {
        let mut weave_builders = HashMap::new();
        let all_weaves = self.all_weaves();
        let mut builder = DrawingFlatsDrawable::new(gpu_spec);
        for (i,weave) in all_weaves.iter().enumerate() {
            weave_builders.insert(weave,(i,PerWeaveFlatPlotAllocator::new(weave)));
        } 
        for (id,request) in self.requests.items() {
            weave_builders.get_mut(&request.weave).unwrap().1.add(id,&request.sizes);
        }
        for weave_builder in weave_builders.values_mut() {
            weave_builder.1.allocate(canvas_store,&mut builder,gpu_spec)?;
        }
        for (id,request) in self.requests.items() {
            let (_,weave_builder) = weave_builders.get(&request.weave).unwrap();
            let origins = weave_builder.origins(&id);
            builder.add(id,weave_builder.canvas.as_ref().unwrap(),origins);
        }
        Ok(builder)
    }
}
