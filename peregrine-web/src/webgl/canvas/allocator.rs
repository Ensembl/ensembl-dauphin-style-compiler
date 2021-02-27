use crate::util::keyed::KeyedData;
use std::collections::{ HashMap, HashSet };
use super::packer::allocate_areas;
use super::store::{ CanvasStore, CanvasElementId };
use super::weave::{ CanvasWeave, DrawingCanvasesBuilder, CanvasRequestId };
use crate::webgl::GPUSpec;

struct CanvasRequest {
    weave: CanvasWeave,
    sizes: Vec<(u32,u32)>
}

struct DrawingCanvasesWeaveBuilderRequest {
    origin: Vec<(u32,u32)>,
    sizes: Vec<(u32,u32)>
}

struct DrawingCanvasesWeaveBuilder {
    requests: HashMap<CanvasRequestId,DrawingCanvasesWeaveBuilderRequest>,
    weave: CanvasWeave,
    canvas: Option<CanvasElementId>
}

impl DrawingCanvasesWeaveBuilder {
    fn new(weave: &CanvasWeave) -> DrawingCanvasesWeaveBuilder {
        DrawingCanvasesWeaveBuilder {
            weave: weave.clone(),
            requests: HashMap::new(),
            canvas: None
        }
    }

    fn add(&mut self, id: CanvasRequestId, sizes: &[(u32,u32)]) {
        self.requests.insert(id.clone(),DrawingCanvasesWeaveBuilderRequest {
            sizes: sizes.to_vec(), origin: vec![]
        });
    }

    fn allocate(&mut self, store: &mut CanvasStore, builder: &mut DrawingCanvasesBuilder, gpuspec: &GPUSpec) -> anyhow::Result<()> {
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

    fn origins(&self, id: &CanvasRequestId) -> Vec<(u32,u32)> {
        self.requests.get(id).unwrap().origin.clone()
    }
}

pub struct DrawingCanvasesAllocator {
    requests: KeyedData<CanvasRequestId,CanvasRequest>
}

impl DrawingCanvasesAllocator {
    pub(crate) fn new() -> DrawingCanvasesAllocator {
        DrawingCanvasesAllocator {
            requests: KeyedData::new()
        }
    }

    pub(crate) fn allocate_areas(&mut self, weave: &CanvasWeave, sizes: &[(u32,u32)]) -> CanvasRequestId {
        self.requests.add(CanvasRequest {
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

    pub(crate) fn make_builder(self, canvas_store: &mut CanvasStore, gpu_spec: &GPUSpec) -> anyhow::Result<DrawingCanvasesBuilder> {
        let mut weave_builders = HashMap::new();
        let all_weaves = self.all_weaves();
        let mut builder = DrawingCanvasesBuilder::new(gpu_spec);
        for (i,weave) in all_weaves.iter().enumerate() {
            weave_builders.insert(weave,(i,DrawingCanvasesWeaveBuilder::new(weave)));
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
