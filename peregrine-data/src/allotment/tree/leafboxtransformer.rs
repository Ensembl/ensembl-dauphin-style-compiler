use crate::{AllotmentMetadataRequest, SpaceBasePointRef, spacebase::spacebase::SpaceBasePoint, CoordinateSystem, allotment::{core::{allotmentmetadata::MetadataMergeStrategy, allotment::Transformer}}};

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct LeafBoxTransformer {
    coord_system: CoordinateSystem,
    secondary: i64,
    top: i64,
    offset: i64,
    size: i64,
    depth: i8,
    reverse: bool
}

impl LeafBoxTransformer {
    pub(crate) fn new(coord_system: &CoordinateSystem, secondary: &Option<i64>, top: i64, offset: i64, size: i64, depth: i8, reverse: bool) -> LeafBoxTransformer {
        LeafBoxTransformer {
            coord_system: coord_system.clone(),
            secondary: secondary.unwrap_or(0).clone(),
            top, offset, size, depth, reverse
        }
    }

    pub(super) fn top(&self) -> i64 { self.top }
    pub(super) fn size(&self) -> i64 { self.size }
}

impl Transformer for LeafBoxTransformer {
    fn transform_spacebase(&self, input: &SpaceBasePointRef<f64>) -> SpaceBasePoint<f64> {
        let mut output = input.make();
        if self.reverse {
            output.normal = (self.offset + self.size) as f64 - output.normal;
        } else {
            output.normal += self.offset as f64;
        }
        output.tangent += self.secondary as f64;
        output
    }

    fn transform_yy(&self, values: &[Option<f64>]) -> Vec<Option<f64>> {
        if self.reverse {
            let offset = (self.offset + self.size) as f64;
            values.iter().map(|x| x.map(|y| offset-y)).collect()
        } else {
            let offset = self.offset as f64;
            values.iter().map(|x| x.map(|y| y+offset)).collect()
        }
    }

    fn add_transform_metadata(&self, out: &mut AllotmentMetadataRequest) {
        let top = self.top();
        let bottom = self.top()+self.size();
        out.add_pair("type","track",&MetadataMergeStrategy::Replace);
        out.add_pair("offset",&top.to_string(),&MetadataMergeStrategy::Minimum);
        out.add_pair("height",&(bottom-top).to_string(),&MetadataMergeStrategy::Maximum);
    }

    fn depth(&self) -> i8 { self.depth }
    fn coord_system(&self) -> CoordinateSystem { self.coord_system.clone() }
}
