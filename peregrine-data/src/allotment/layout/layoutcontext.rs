use crate::{globals::trainstate::CarriageTrainStateRequest, shapeload::shaperequestgroup::ShapeRequestGroup};

pub(crate) struct LayoutContext {
    pub(crate) extent: Option<ShapeRequestGroup>,
    pub(crate) state_request: CarriageTrainStateRequest,
}

impl LayoutContext {
    pub(crate) fn new(extent: Option<&ShapeRequestGroup>) -> LayoutContext {
        LayoutContext {
            extent: extent.cloned(),
            state_request: CarriageTrainStateRequest::new(),
        }
    }
}
