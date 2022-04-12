use std::{collections::HashMap, sync::{Arc, Mutex}};

use peregrine_toolkit::puzzle::AnswerAllocator;

use crate::{allotment::{style::{allotmentname::{AllotmentName, BuildPassThroughHasher, new_efficient_allotmentname_hashmap}, stylebuilder::make_transformable}, util::bppxconverter::BpPxConverter}, LeafRequest, ShapeRequestGroup, DataMessage};

use super::{carriageoutput::BoxPositionContext, trainstate::CarriageTrainStateSpec};

pub struct LeafList {
    leafs: HashMap<AllotmentName,LeafRequest,BuildPassThroughHasher>
}

impl LeafList {
    pub fn new() -> LeafList {
        LeafList {
            leafs: new_efficient_allotmentname_hashmap()
        }
    }

    pub fn pending_leaf(&mut self, spec: &str) -> &mut LeafRequest {
        let name = AllotmentName::new(spec);
        if !self.leafs.contains_key(&name) {
            self.leafs.insert(name.clone(),LeafRequest::new(&AllotmentName::new(spec)));
        }
        self.leafs.get_mut(&name).unwrap()
    }

    pub fn union(&self, other: &LeafList) -> LeafList {
        let mut leafs = self.leafs.clone();
        leafs.extend(&mut other.leafs.iter().map(|(k,v)| (k.clone(),v.clone())));
        LeafList {
            leafs
        }
    }

    pub(super) fn position_boxes(&self, extent: Option<&ShapeRequestGroup>, answer_allocator: &Arc<Mutex<AnswerAllocator>>) -> Result<(BoxPositionContext,CarriageTrainStateSpec),DataMessage> {
        let mut prep = BoxPositionContext::new(extent,answer_allocator);
        let spec = make_transformable(&mut prep,&mut self.leafs.values())?;
        Ok((prep,spec))
    }
}
