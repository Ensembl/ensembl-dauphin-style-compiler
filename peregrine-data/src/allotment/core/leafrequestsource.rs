use std::sync::Arc;
use peregrine_toolkit::error::Error;
use crate::{LeafRequest, ShapeRequestGroup, allotment::{layout::stylebuilder::make_transformable, leafs::floating::FloatingLeaf}, shape::{metadata::AbstractMetadata}, Shape, globals::trainstate::CarriageTrainStateSpec};
use super::{allotmentname::{allotmentname_hashmap, AllotmentName, AllotmentNameHashMap}, boxpositioncontext::BoxPositionContext };

pub(crate) struct LeafRequestSource {
    leafs: AllotmentNameHashMap<LeafRequest>
}

impl LeafRequestSource {
    pub(crate) fn new() -> LeafRequestSource {
        LeafRequestSource {
            leafs: allotmentname_hashmap()
        }
    }

    pub(crate) fn merge(input: Vec<Arc<LeafRequestSource>>) -> LeafRequestSource {
        let len = input.iter().map(|x| x.leafs.len()).sum();
        let mut leafs = allotmentname_hashmap();
        leafs.reserve(len);
        for more in input {
            leafs.extend(more.leafs.iter().map(|(n,r)| (n.clone(),r.clone())));
        }
        LeafRequestSource { leafs }
    }

    pub(crate) fn pending_leaf(&mut self, spec: &str) -> &mut LeafRequest {
        let name = AllotmentName::new(spec);
        if !self.leafs.contains_key(&name) {
            self.leafs.insert(name.clone(),LeafRequest::new(&AllotmentName::new(spec)));
        }
        self.leafs.get_mut(&name).unwrap()
    }

    pub(super) fn to_floating_shapes(&self, shapes: &[Shape<LeafRequest>], extent: Option<&ShapeRequestGroup>, metadata: &AbstractMetadata) -> Result<(CarriageTrainStateSpec,Vec<Shape<FloatingLeaf>>),Error> {
        /* makes the layout tree */
        let (mut root,plm) = make_transformable(&mut self.leafs.values())?;
        /* runs the up-and-down algorithm to place tree boxes */
        let mut prep = BoxPositionContext::new(extent,metadata);
        let spec = root.full_build(&mut prep);
        /* Maps shapes to new FloatingLeafs */
        let shapes = shapes.iter().map(|x| 
            x.map_new_allotment(|r| plm.get(r.name()).unwrap().clone())
        ).collect::<Vec<_>>();
        Ok((spec,shapes))
    }
}