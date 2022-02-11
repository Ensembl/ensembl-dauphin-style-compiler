use crate::{AllotmentMetadataRequest, SpaceBasePointRef, spacebase::{spacebase::SpaceBasePoint, spacebase2::SpaceBase2PointRef}, CoordinateSystem, allotment::{core::{allotmentmetadata::MetadataMergeStrategy, allotment::Transformer}}, SpaceBase, Allotment, SpaceBase2Point, SpaceBase2, SpaceBaseArea2, PartialSpaceBase2};

use super::allotmentbox::AllotmentBox;

pub fn transform_spacebase2(coord_system: &CoordinateSystem, input: &SpaceBase2<f64,Allotment>) -> SpaceBase2<f64,Allotment> {
    let mut output = input.clone();
    if coord_system.up_from_bottom() {
        output.update_normal_from_allotment(|n,a| { *n = (a.allotment_box().draw_bottom() as f64) - *n; });
    } else {
        output.update_normal_from_allotment(|n,a| { *n += a.allotment_box().draw_top() as f64; });
    }
    output.update_tangent_from_allotment(|t,a| { *t += a.allotment_box().indent() as f64; });
    output
}

pub fn transform_spacebasearea2(coord_system: &CoordinateSystem, input: &SpaceBaseArea2<f64,Allotment>) -> SpaceBaseArea2<f64,Allotment> {
    let top_left = transform_spacebase2(coord_system,input.top_left());
    let bottom_right = transform_spacebase2(coord_system,&input.bottom_right());
    SpaceBaseArea2::new(PartialSpaceBase2::from_spacebase(top_left),PartialSpaceBase2::from_spacebase(bottom_right)).unwrap()
}

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
    fn transform_spacebase(&self, mut input: SpaceBase<f64>) -> SpaceBase<f64> {
        if self.geometry.up_from_bottom() {
            let bottom =  self.allot_box.draw_bottom() as f64;
            input.update_normal(|n| { *n = bottom-*n; });
        } else {
            let top = self.allot_box.draw_top() as f64;
            input.update_normal(|n| { *n += top; });
        }
        let indent = self.allot_box.indent() as f64;
        input.update_tangent(|t| { *t += indent });
        input
    }

    fn transform_spacebase_point(&self, input: &SpaceBasePointRef<f64>) -> SpaceBasePoint<f64> {
        let mut output = input.make();
        if self.geometry.up_from_bottom() {
            output.normal = self.allot_box.draw_bottom() as f64 - output.normal;
        } else {
            output.normal += self.allot_box.draw_top() as f64;
        }
        output.tangent += self.allot_box.indent() as f64;
        output
    }

    fn transform_spacebase2_point(&self, input: &SpaceBase2PointRef<f64,Allotment>) -> SpaceBase2Point<f64,Allotment> {
        let mut output = input.make();
        if self.geometry.up_from_bottom() {
            output.normal = self.allot_box.draw_bottom() as f64 - output.normal;
        } else {
            output.normal += self.allot_box.draw_top() as f64;
        }
        output.tangent += self.allot_box.indent() as f64;
        output
    }

    fn transform_spacebase2(&self, input: &SpaceBase2<f64,Allotment>) -> SpaceBase2<f64,Allotment> {
        let mut output = input.clone();
        if self.geometry.up_from_bottom() {
            let bottom =  self.allot_box.draw_bottom() as f64;
            output.update_normal(|n| { *n = bottom-*n; });
        } else {
            let top = self.allot_box.draw_top() as f64;
            output.update_normal(|n| { *n += top; });
        }
        let indent = self.allot_box.indent() as f64;
        output.update_tangent(|t| { *t += indent });
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
