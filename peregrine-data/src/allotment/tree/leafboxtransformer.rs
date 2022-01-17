use crate::{AllotmentMetadataRequest, SpaceBasePointRef, spacebase::spacebase::SpaceBasePoint, CoordinateSystem, allotment::{core::{allotmentmetadata::MetadataMergeStrategy, allotment::Transformer}}};

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

    pub fn coord_system(&self) -> CoordinateSystem { self.coord_system.clone() }
}

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct LeafBoxTransformer {
    geometry: LeafGeometry,
    secondary: i64,
    offset: i64,
    size: i64,
    depth: i8,
}

impl LeafBoxTransformer {
    pub(crate) fn new(geometry: &LeafGeometry, secondary: &Option<i64>, offset: i64, size: i64, depth: i8) -> LeafBoxTransformer {
        LeafBoxTransformer {
            geometry: geometry.clone(),
            secondary: secondary.unwrap_or(0).clone(),
            offset, size, depth
        }
    }
}

impl Transformer for LeafBoxTransformer {
    fn transform_spacebase(&self, input: &SpaceBasePointRef<f64>) -> SpaceBasePoint<f64> {
        let mut output = input.make();
        if self.geometry.reverse {
            output.normal = (self.offset + self.size) as f64 - output.normal;
        } else {
            output.normal += self.offset as f64;
        }
        output.tangent += self.secondary as f64;
        output
    }

    fn transform_yy(&self, values: &[Option<f64>]) -> Vec<Option<f64>> {
        if self.geometry.reverse {
            let offset = (self.offset + self.size) as f64;
            values.iter().map(|x| x.map(|y| offset-y)).collect()
        } else {
            let offset = self.offset as f64;
            values.iter().map(|x| x.map(|y| y+offset)).collect()
        }
    }

    fn add_transform_metadata(&self, out: &mut AllotmentMetadataRequest) {
        out.add_pair("type","track",&MetadataMergeStrategy::Replace);
        out.add_pair("offset",&self.offset.to_string(),&MetadataMergeStrategy::Minimum);
        out.add_pair("height",&self.size.to_string(),&MetadataMergeStrategy::Maximum);
    }

    fn depth(&self) -> i8 { self.depth }
    fn coord_system(&self) -> CoordinateSystem { self.geometry.coord_system.clone() }
}
