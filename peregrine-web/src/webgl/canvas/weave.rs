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
    origin: Vec<(u32,u32)>
}

// TODO not pub
pub struct CanvasTextureAreas {
    pub texture_origin: (u32,u32),
    pub mask_origin: (u32,u32),
    pub size: (u32,u32)
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

    pub(super) fn add(&mut self, id: CanvasRequestId, canvas: &CanvasElementId, origin: Vec<(u32,u32)>) {
        self.areas.insert(&id,CanvasArea {
            canvas: canvas.clone(),
            origin
        });
    }

    pub(super) fn make_canvas(&mut self, store: &mut CanvasStore, weave: &CanvasWeave, size: (u32,u32)) -> anyhow::Result<CanvasElementId> {
        self.canvases.allocate_main(store,weave,size)
    }

    pub(crate) fn gl_index(&self, id: &CanvasElementId) -> anyhow::Result<usize> {
        self.canvases.gl_index(id)
    }

    // TODO merge
    pub(crate) fn origins(&self, id: &CanvasRequestId) -> Vec<(u32,u32)> {
        self.areas.get(id).as_ref().map(|a| &a.origin).unwrap().to_vec()
    }

    pub(crate) fn canvas(&self, id: &CanvasRequestId) -> CanvasElementId {
        self.areas.get(id).as_ref().map(|a| a.canvas.clone()).unwrap()
    }

    pub(crate) fn built(self) -> DrawingCanvases { self.canvases }
}
