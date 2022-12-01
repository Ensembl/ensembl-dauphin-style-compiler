use std::sync::Arc;

use peregrine_toolkit::error::Error;

use crate::{LeafRequest, ShapeRequestGroup, allotment::builder::stylebuilder::make_transformable, shape::metadata::AbstractMetadata};
use super::{trainstate::CarriageTrainStateSpec, allotmentname::{allotmentname_hashmap, AllotmentName, AllotmentNameHashMap}, boxpositioncontext::BoxPositionContext, leafrequest::LeafTransformableMap};


pub struct LeafList {
    leafs: AllotmentNameHashMap<LeafRequest>
}

impl LeafList {
    pub fn new() -> LeafList {
        LeafList {
            leafs: allotmentname_hashmap()
        }
    }

    pub(crate) fn merge(input: Vec<Arc<LeafList>>) -> LeafList {
        let len = input.iter().map(|x| x.leafs.len()).sum();
        let mut leafs = allotmentname_hashmap();
        leafs.reserve(len);
        for more in input {
            leafs.extend(more.leafs.iter().map(|(n,r)| (n.clone(),r.clone())));
        }
        LeafList { leafs }
    }

    pub fn pending_leaf(&mut self, spec: &str) -> &mut LeafRequest {
        let name = AllotmentName::new(spec);
        if !self.leafs.contains_key(&name) {
            self.leafs.insert(name.clone(),LeafRequest::new(&AllotmentName::new(spec)));
        }
        self.leafs.get_mut(&name).unwrap()
    }

    pub(super) fn position_boxes(&self, extent: Option<&ShapeRequestGroup>, metadata: &AbstractMetadata) -> Result<(CarriageTrainStateSpec,LeafTransformableMap),Error> {
        let mut prep = BoxPositionContext::new(extent,metadata);
        let (spec,plm) = make_transformable(&mut prep,&mut self.leafs.values())?;
        Ok((spec,plm))
    }
}
