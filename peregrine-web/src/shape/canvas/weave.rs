use super::store::{ CanvasStore, DrawingCanvases, CanvasElementId };
use crate::util::keyed::KeyedData;
use crate::webgl::GPUSpec;
use crate::keyed_handle;

#[derive(Clone,PartialEq,Eq,Hash)]
pub enum CanvasWeave {
    Crisp,
    Fuzzy
}

keyed_handle!(CanvasRequestId);

pub struct CanvasArea {
    canvas: CanvasElementId,
    origin: (u32,u32),
    size: (u32,u32)
}

pub struct DrawingCanvasesBuilder {
    areas: KeyedData<CanvasRequestId,Option<CanvasArea>>,
    canvases: DrawingCanvases
}

impl DrawingCanvasesBuilder {
    pub(super) fn new(gpuspec: &GPUSpec) -> DrawingCanvasesBuilder {
        DrawingCanvasesBuilder {
            areas: KeyedData::new(),
            canvases: DrawingCanvases::new(gpuspec)
        }
    }

    pub(super) fn add(&mut self, id: CanvasRequestId, canvas: &CanvasElementId, origin: (u32,u32), size: (u32,u32)) {
        self.areas.insert(&id,CanvasArea {
            canvas: canvas.clone(),
            origin, size
        });
    }

    pub(super) fn make_canvas(&mut self, store: &mut CanvasStore, weave: &CanvasWeave, size: (u32,u32)) -> anyhow::Result<CanvasElementId> {
        self.canvases.allocate_main(store,weave,size)
    }

    // TODO merge
    pub(super) fn origin(&self, id: &CanvasRequestId) -> (u32,u32) {
        self.areas.get(id).as_ref().map(|a| a.origin).unwrap()
    }

    pub(super) fn size(&self, id: &CanvasRequestId) -> (u32,u32) {
        self.areas.get(id).as_ref().map(|a| a.size).unwrap()
    }

    pub(super) fn canvas(&self, id: &CanvasRequestId) -> CanvasElementId {
        self.areas.get(id).as_ref().map(|a| a.canvas.clone()).unwrap()
    }

    pub(crate) fn built(self) -> DrawingCanvases { self.canvases }
}
