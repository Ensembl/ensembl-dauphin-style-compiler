use std::{collections::HashMap, sync::Arc};

use peregrine_toolkit::puzzle::PuzzleBuilder;

use crate::{allotment::{style::{allotmentname::{AllotmentName, BuildPassThroughHasher, new_efficient_allotmentname_hashmap}, stylebuilder::make_transformable}, util::bppxconverter::BpPxConverter}, LeafRequest, ShapeRequestGroup, DataMessage};

use super::carriageoutput::CarriageUniversePrep;

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

    pub(super) fn make_transformable(&self, extent: Option<&ShapeRequestGroup>) -> Result<CarriageUniversePrep,DataMessage> {
        let mut builder = PuzzleBuilder::new();
        let mut prep = CarriageUniversePrep::new(&mut builder,extent.map(|x| x.region().clone()));
        let bp_px_converter = Arc::new(BpPxConverter::new(extent));
        make_transformable(&mut prep,&bp_px_converter,&mut self.leafs.values())?;
        Ok(prep)
    }
}
