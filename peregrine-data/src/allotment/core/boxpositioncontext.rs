use std::sync::Arc;
use crate::{allotment::{util::bppxconverter::BpPxConverter, collision::collisionalgorithm::BumpRequestSetFactory}, ShapeRequestGroup, shape::metadata::AbstractMetadata};
use super::{trainstate::CarriageTrainStateRequest};

pub(crate) struct BoxPositionContext {
    pub(crate) bp_px_converter: Arc<BpPxConverter>,
    pub(crate) state_request: CarriageTrainStateRequest,
    pub(crate) bumper_factory: BumpRequestSetFactory
}

impl BoxPositionContext {
    pub(crate) fn new(extent: Option<&ShapeRequestGroup>, metadata: &AbstractMetadata) -> BoxPositionContext {
        let index = extent.map(|x| x.region().index()).unwrap_or(0);
        let bumper_factory = BumpRequestSetFactory::new(index as usize);
        BoxPositionContext {
            bp_px_converter: Arc::new(BpPxConverter::new(extent)),
            state_request: CarriageTrainStateRequest::new(metadata),
            bumper_factory
        }
    }
}
