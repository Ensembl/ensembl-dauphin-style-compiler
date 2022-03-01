use std::sync::Arc;

use crate::{CoordinateSystem, SpaceBase, SpaceBaseArea, PartialSpaceBase};

pub trait SpaceBaseTransformer {
    type X;

    fn transform_spacebase(coord_system: &CoordinateSystem, input: &SpaceBase<f64,Self::X>) -> SpaceBase<f64,()>;

    fn transform_spacebasearea(coord_system: &CoordinateSystem, input: &SpaceBaseArea<f64,Self::X>) -> SpaceBaseArea<f64,()> {
        let top_left = Self::transform_spacebase(coord_system,input.top_left());
        let bottom_right = Self::transform_spacebase(coord_system,&input.bottom_right());
        SpaceBaseArea::new(PartialSpaceBase::from_spacebase(top_left),PartialSpaceBase::from_spacebase(bottom_right)).unwrap()
    }    
}


pub trait GraphTransformer {
    type X;

    fn transform_yy(coord_system: &CoordinateSystem, allot_box: &Self::X, values: &[Option<f64>]) -> Vec<Option<f64>>;
}
