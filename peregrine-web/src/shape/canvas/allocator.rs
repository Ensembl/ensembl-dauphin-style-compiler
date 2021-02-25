use crate::util::keyed::KeyedData;
use std::collections::{ HashMap, HashSet };
use super::packer::allocate_areas;
use super::store::{ CanvasStore, CanvasElementId, DrawingCanvases };
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
    sizes: Vec<(u32,u32)>,
    canvas: Option<CanvasElementId>,
}

impl DrawingCanvasesWeaveBuilder {
    fn new(weave: &CanvasWeave) -> DrawingCanvasesWeaveBuilder {
        DrawingCanvasesWeaveBuilder {
            weave: weave.clone(),
            ids: vec![],
            origins: HashMap::new(),
            sizes: vec![],
            canvas: None
        }
    }

    fn add(&mut self, id: CanvasRequestId, size: (u32,u32)) {
        self.ids.push(id);
        self.sizes.push(size);
    }

    fn allocate(&mut self, store: &mut CanvasStore, builder: &mut DrawingCanvasesBuilder, gpuspec: &GPUSpec) -> anyhow::Result<()> {
        let (origins,width,height) = allocate_areas(&self.sizes,gpuspec)?;
        for (i,id) in self.ids.iter().enumerate() {
            self.origins.insert(id.clone(),origins[i]);
        }
        self.canvas = Some(builder.make_canvas(store,&self.weave,(width,height))?);
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
    pub(crate) fn new() -> DrawingCanvasesAllocator {
        DrawingCanvasesAllocator {
            requests: KeyedData::new()
        }
    }

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

    pub(crate) fn make_builder(self, canvas_store: &mut CanvasStore, gpu_spec: &GPUSpec) -> anyhow::Result<DrawingCanvasesBuilder> {
        let mut weave_builders = HashMap::new();
        let all_weaves = self.all_weaves();
        let mut builder = DrawingCanvasesBuilder::new(gpu_spec);
        for (i,weave) in all_weaves.iter().enumerate() {
            weave_builders.insert(weave,(i,DrawingCanvasesWeaveBuilder::new(weave)));
        } 
        for (id,request) in self.requests.items() {
            weave_builders.get_mut(&request.weave).unwrap().1.add(id,request.size);
        }
        for weave_builder in weave_builders.values_mut() {
            weave_builder.1.allocate(canvas_store,&mut builder,gpu_spec)?;
        }
        for (id,request) in self.requests.items() {
            let (canvas_idx,weave_builder) = weave_builders.get(&request.weave).unwrap();
            let origin = weave_builder.origin(&id);
            builder.add(id,weave_builder.canvas.as_ref().unwrap(),origin,request.size);
        }
        Ok(builder)
    }
}
