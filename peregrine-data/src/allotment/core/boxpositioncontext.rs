use std::sync::Arc;

use crate::{allotment::{util::bppxconverter::BpPxConverter, boxes::root::Root, collision::collisionalgorithm::BumpRequestSetFactory}, ShapeRequestGroup};

use super::{leafrequest::LeafTransformableMap, trainstate::CarriageTrainStateRequest};

pub(crate) struct BoxPositionContext {
    pub bp_px_converter: Arc<BpPxConverter>,
    pub root: Root,
    pub plm: LeafTransformableMap,
    pub state_request: CarriageTrainStateRequest,
    pub bumper_factory: BumpRequestSetFactory
}

impl BoxPositionContext {
    pub(crate) fn new(extent: Option<&ShapeRequestGroup>) -> BoxPositionContext {
        let index = extent.map(|x| x.region().index()).unwrap_or(0);
        let bumper_factory = BumpRequestSetFactory::new(index as usize);
        BoxPositionContext {
            bp_px_converter: Arc::new(BpPxConverter::new(extent)),
            root: Root::new(),
            plm: LeafTransformableMap::new(),
            state_request: CarriageTrainStateRequest::new(),
            bumper_factory
        }
    }
}
