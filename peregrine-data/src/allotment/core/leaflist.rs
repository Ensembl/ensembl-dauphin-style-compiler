use crate::{LeafRequest, ShapeRequestGroup, DataMessage, allotment::builder::stylebuilder::make_transformable};
use super::{carriageoutput::BoxPositionContext, trainstate::CarriageTrainStateSpec, allotmentname::{allotmentname_hashmap, AllotmentName, AllotmentNameHashMap}};

pub struct LeafList {
    leafs: AllotmentNameHashMap<LeafRequest>
}

impl LeafList {
    pub fn new() -> LeafList {
        LeafList {
            leafs: allotmentname_hashmap()
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

    pub(super) fn position_boxes(&self, extent: Option<&ShapeRequestGroup>) -> Result<(BoxPositionContext,CarriageTrainStateSpec),DataMessage> {
        let mut prep = BoxPositionContext::new(extent);
        let spec = make_transformable(&mut prep,&mut self.leafs.values())?;
        Ok((prep,spec))
    }
}
