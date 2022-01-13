use crate::{AllotmentMetadata, AllotmentMetadataRequest, SpaceBasePointRef, spacebase::spacebase::SpaceBasePoint};
use super::{allotment::{AllotmentImpl, CoordinateSystem}};

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct LeafBoxAllotment {
    metadata: AllotmentMetadata,
    coord_system: CoordinateSystem,
    secondary: i64,
    top: i64,
    offset: i64,
    size: i64,
    depth: i8,
    secret: bool,
    reverse: bool
}

impl LeafBoxAllotment {
    pub(crate) fn new(coord_system: &CoordinateSystem, metadata: &AllotmentMetadata, secondary: i64, top: i64, offset: i64, size: i64, depth: i8, reverse: bool) -> LeafBoxAllotment {
        let secret = metadata.get_i64("secret-track").unwrap_or(0) != 0;
        LeafBoxAllotment {
            coord_system: coord_system.clone(),
            metadata: metadata.clone(),
            secondary, top, offset, size, depth, secret, reverse
        }
    }

    pub(super) fn add_metadata(&self, full_metadata: &mut AllotmentMetadataRequest) {
        if !self.secret {
            full_metadata.add_pair("type","track");
            full_metadata.add_pair("offset",&self.top.to_string());
            full_metadata.add_pair("height",&self.size.to_string());
        }
    }
}

impl AllotmentImpl for LeafBoxAllotment {
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

    fn depth(&self) -> i8 { self.depth }

    fn coord_system(&self) -> CoordinateSystem { self.coord_system.clone() }
}
