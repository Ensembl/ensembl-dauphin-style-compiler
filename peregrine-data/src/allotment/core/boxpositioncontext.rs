use crate::{ShapeRequestGroup, globals::trainstate::CarriageTrainStateRequest};

pub(crate) struct BoxPositionContext {
    pub(crate) extent: Option<ShapeRequestGroup>,
    pub(crate) state_request: CarriageTrainStateRequest,
}

impl BoxPositionContext {
    pub(crate) fn new(extent: Option<&ShapeRequestGroup>) -> BoxPositionContext {
        BoxPositionContext {
            extent: extent.cloned(),
            state_request: CarriageTrainStateRequest::new(),
        }
    }
}
