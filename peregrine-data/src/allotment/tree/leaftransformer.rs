use crate::{AllotmentMetadataRequest, SpaceBasePointRef, spacebase::spacebase::SpaceBasePoint, CoordinateSystem, allotment::{core::{allotmentmetadata::MetadataMergeStrategy, allotment::Transformer}}};

use super::allotmentbox::AllotmentBox;

pub struct LeafTransformer {
    geometry: CoordinateSystem,
    allot_box: AllotmentBox,
}

impl LeafTransformer {
    pub(crate) fn new(geometry: &CoordinateSystem, allot_box: &AllotmentBox) -> LeafTransformer {
        LeafTransformer {
            geometry: geometry.clone(),
            allot_box: allot_box.clone()
        }
    }
}

impl Transformer for LeafTransformer {
    fn transform_spacebase(&self, input: &SpaceBasePointRef<f64>) -> SpaceBasePoint<f64> {
        let mut output = input.make();
        if self.geometry.up_from_bottom() {
            output.normal = self.allot_box.draw_bottom() as f64 - output.normal;
        } else {
            output.normal += self.allot_box.draw_top() as f64;
        }
        output.tangent += self.allot_box.indent() as f64;
        output
    }

    fn transform_yy(&self, values: &[Option<f64>]) -> Vec<Option<f64>> {
        if self.geometry.up_from_bottom() {
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
}
