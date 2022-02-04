use crate::{AllotmentMetadataRequest, SpaceBasePointRef, spacebase::spacebase::SpaceBasePoint, CoordinateSystem, allotment::{core::{allotmentmetadata::MetadataMergeStrategy, allotment::Transformer}}};

use super::allotmentbox::AllotmentBox;

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
pub struct LeafGeometry {
    coord_system: CoordinateSystem,
    reverse: bool
}

impl LeafGeometry {
    pub fn new( coord_system: CoordinateSystem, reverse: bool) -> LeafGeometry {
        LeafGeometry { coord_system, reverse }
    }

    pub fn with_new_coord_system(&self, coord_system: &CoordinateSystem) -> LeafGeometry {
        LeafGeometry {
            coord_system: coord_system.clone(),
            reverse: self.reverse
        }
    }

    pub fn reverse(&self) -> bool { self.reverse }
    pub fn coord_system(&self) -> CoordinateSystem { self.coord_system.clone() }
}

pub struct LeafTransformer {
    geometry: LeafGeometry,
    allot_box: AllotmentBox,
    depth: i8,
}

impl LeafTransformer {
    pub(crate) fn new(geometry: &LeafGeometry, allot_box: &AllotmentBox, depth: i8) -> LeafTransformer {
        LeafTransformer {
            geometry: geometry.clone(),
            allot_box: allot_box.clone(),
            depth
        }
    }
}

impl Transformer for LeafTransformer {
    fn transform_spacebase(&self, input: &SpaceBasePointRef<f64>) -> SpaceBasePoint<f64> {
        let mut output = input.make();
        if self.geometry.reverse {
            output.normal = self.allot_box.draw_bottom() as f64 - output.normal;
        } else {
            output.normal += self.allot_box.draw_top() as f64;
        }
        output.tangent += self.allot_box.indent() as f64;
        output
    }

    fn transform_yy(&self, values: &[Option<f64>]) -> Vec<Option<f64>> {
        if self.geometry.reverse {
            let offset = self.allot_box.draw_bottom() as f64;
            values.iter().map(|x| x.map(|y| offset-y)).collect()
        } else {
            let offset = self.allot_box.draw_top() as f64;
            values.iter().map(|x| x.map(|y| y+offset)).collect()
        }
    }

    fn add_transform_metadata(&self, out: &mut AllotmentMetadataRequest) {
        out.add_pair("type","track",&MetadataMergeStrategy::Replace);
        out.add_pair("offset",&self.allot_box.top().to_string(),&MetadataMergeStrategy::Minimum);
        out.add_pair("height",&(self.allot_box.bottom()-self.allot_box.top()).to_string(),&MetadataMergeStrategy::Maximum);
    }

    fn depth(&self) -> i8 { self.depth }
    fn coord_system(&self) -> CoordinateSystem { self.geometry.coord_system.clone() }
}