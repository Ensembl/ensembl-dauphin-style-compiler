use crate::util::keyed::KeyedData;
use std::collections::{ HashMap, HashSet };
use super::packer::allocate_areas;
use super::weave::{ CanvasWeave, DrawingCanvasesBuilder, CanvasRequestId };
use crate::webgl::GPUSpec;

struct CanvasRequest {
    weave: CanvasWeave,
    size: (u32,u32)
}

struct DrawingCanvasesWeaveBuilder {
    weave: CanvasWeave,
    ids: Vec<CanvasRequestId>,
    origins: HashMap<CanvasRequestId,(u32,u32)>,
    sizes: Vec<(u32,u32)>
}

impl DrawingCanvasesWeaveBuilder {
    fn new(weave: &CanvasWeave) -> DrawingCanvasesWeaveBuilder {
        DrawingCanvasesWeaveBuilder {
            weave: weave.clone(),
            ids: vec![],
            origins: HashMap::new(),
            sizes: vec![]
        }
    }

    fn add(&mut self, id: CanvasRequestId, size: (u32,u32)) {
        self.ids.push(id);
        self.sizes.push(size);
    }

    fn calculate(&mut self, gpuspec: &GPUSpec) -> anyhow::Result<()> {
        let origins = allocate_areas(&self.sizes,gpuspec)?;
        for (i,id) in self.ids.iter().enumerate() {
            self.origins.insert(id.clone(),origins[i]);
        }
        Ok(())
    }

    fn origin(&self, id: &CanvasRequestId) -> (u32,u32) {
        self.origins.get(id).unwrap().clone()
    }
}

pub struct DrawingCanvasesAllocator {
    requests: KeyedData<CanvasRequestId,CanvasRequest>
}

impl DrawingCanvasesAllocator {
    pub(crate) fn allocate_area(&mut self, weave: &CanvasWeave, size: (u32,u32)) -> anyhow::Result<CanvasRequestId> {
        Ok(self.requests.add(CanvasRequest {
            weave: weave.clone(),
            size
        }))
    }

    fn all_weaves(&self) -> Vec<CanvasWeave> {
        let mut out = HashSet::new();
        for request in self.requests.values() {
            out.insert(request.weave.clone());
        }
        out.iter().cloned().collect()
    }

    pub(crate) fn make_builder(self, gpu_spec: &GPUSpec) -> anyhow::Result<DrawingCanvasesBuilder> {
        let mut weave_builders = HashMap::new();
        let all_weaves = self.all_weaves();
        let mut builder = DrawingCanvasesBuilder::new();
        let canvases : Vec<_> = all_weaves.iter().map(|w| builder.make_canvas(w)).collect();
        for (i,weave) in all_weaves.iter().enumerate() {
            weave_builders.insert(weave,(i,DrawingCanvasesWeaveBuilder::new(weave)));
        } 
        for (id,request) in self.requests.items() {
            weave_builders.get_mut(&request.weave).unwrap().1.add(id,request.size);
        }
        for weave_builder in weave_builders.values_mut() {
            weave_builder.1.calculate(gpu_spec)?;
        }
        for (id,request) in self.requests.items() {
            let (canvas_idx,weave_builder) = weave_builders.get(&request.weave).unwrap();
            let origin = weave_builder.origin(&id);
            builder.add(id,&canvases[*canvas_idx],origin,request.size);
        }
        Ok(builder)
    }
}
