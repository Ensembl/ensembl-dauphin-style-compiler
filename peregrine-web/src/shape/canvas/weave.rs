use crate::util::keyed::KeyedData;
use crate::keyed_handle;

#[derive(Clone,PartialEq,Eq,Hash)]
pub enum CanvasWeave {
    Crisp,
    Fuzzy
}

#[derive(Clone,PartialEq,Eq,Hash)]
pub struct CanvasInstanceId {
    weave: CanvasWeave,
    //instance: usize
}

keyed_handle!(CanvasInstanceHandle);
keyed_handle!(CanvasRequestId);

pub struct CanvasInstance {
    id: CanvasInstanceId
}

impl CanvasInstance {
    pub(super) fn new(weave: &CanvasWeave) -> CanvasInstance {
        CanvasInstance {
            id: CanvasInstanceId {
                weave: weave.clone()
            }
        }
    }
}

pub struct CanvasArea {
    canvas: CanvasInstanceHandle,
    origin: (u32,u32),
    size: (u32,u32)
}

pub struct DrawingCanvasesBuilder {
    areas: KeyedData<CanvasRequestId,Option<CanvasArea>>,
    canvases: KeyedData<CanvasInstanceHandle,CanvasInstance>
}

impl DrawingCanvasesBuilder {
    pub(super) fn new() -> DrawingCanvasesBuilder {
        DrawingCanvasesBuilder {
            areas: KeyedData::new(),
            canvases: KeyedData::new()
        }
    }

    pub(super) fn make_canvas(&mut self, weave: &CanvasWeave) -> CanvasInstanceHandle {
        self.canvases.add(CanvasInstance::new(weave))
    }

    pub(super) fn add(&mut self, id: CanvasRequestId, canvas: &CanvasInstanceHandle, origin: (u32,u32), size: (u32,u32)) {
        self.areas.insert(&id,CanvasArea {
            canvas: canvas.clone(),
            origin, size
        });
    }
}
