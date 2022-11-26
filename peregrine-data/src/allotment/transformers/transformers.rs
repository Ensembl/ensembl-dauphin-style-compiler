use crate::{SpaceBase, SpaceBaseArea, allotment::{style::style::{LeafStyle}, boxes::leaf::AnchoredLeaf}, CoordinateSystem};
use super::{transformertraits::{SpaceBaseTransformer, GraphTransformer}, simple::SimpleTransformerHolder};

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
#[derive(Hash,PartialEq,Eq)]
pub enum TransformerVariety {
    DustbinTransformer,
    SimpleTransformer
}

impl TransformerVariety {
    pub fn spacebase_transform(&self, coord_system: &CoordinateSystem, spacebase: &SpaceBase<f64,AnchoredLeaf>) -> SpaceBase<f64,LeafStyle> {
        match self {
            TransformerVariety::SimpleTransformer => {
                let items = spacebase.map_allotments(|a| a.into_simple_transformer());
                SimpleTransformerHolder::transform_spacebase(coord_system,&items)
            },
            TransformerVariety::DustbinTransformer => { spacebase.map_allotments(|_| LeafStyle::dustbin()) }
        }
    }

    pub fn spacebasearea_transform(&self, coord_system: &CoordinateSystem, spacebase: &SpaceBaseArea<f64,AnchoredLeaf>) -> SpaceBaseArea<f64,LeafStyle> {
        match self {
            TransformerVariety::SimpleTransformer => {
                let items = spacebase.map_allotments(|a| a.into_simple_transformer());
                SimpleTransformerHolder::transform_spacebasearea(coord_system,&items)
            },
            TransformerVariety::DustbinTransformer => { spacebase.map_allotments(|_| LeafStyle::dustbin()) }
        }
    }

    pub fn graph_transform(&self, coord_system: &CoordinateSystem, allot_box: &AnchoredLeaf, values: &[Option<f64>]) -> Vec<Option<f64>> {
        match self {
            TransformerVariety::SimpleTransformer => {
                SimpleTransformerHolder::transform_yy(coord_system,allot_box.into_simple_transformer(),values)
            },
            TransformerVariety::DustbinTransformer => { values.to_vec() }
        }
    }
}
