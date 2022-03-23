use std::sync::Arc;

use crate::{CoordinateSystem, SpaceBase, SpaceBaseArea, allotment::style::style::{LeafCommonStyle}};

use super::{transformertraits::{SpaceBaseTransformer, GraphTransformer}, simple::SimpleTransformerHolder};

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
#[derive(Hash,PartialEq,Eq)]
pub enum TransformerVariety {
    DustbinTransformer,
    SimpleTransformer
}

pub trait Transformer {
    fn choose_variety(&self) -> (TransformerVariety,CoordinateSystem);
    fn into_simple_transformer(&self) -> Option<SimpleTransformerHolder> { None }
    fn get_style(&self) -> &LeafCommonStyle;

    #[cfg(any(debug_assertions,test))]
    fn describe(&self) -> String;
}

impl TransformerVariety {
    pub fn spacebase_transform(&self, coord_system: &CoordinateSystem, spacebase: &SpaceBase<f64,Arc<dyn Transformer>>) -> SpaceBase<f64,LeafCommonStyle> {
        match self {
            TransformerVariety::SimpleTransformer => {
                let items = spacebase.map_allotments(|a| a.into_simple_transformer());
                SimpleTransformerHolder::transform_spacebase(coord_system,&items)
            },
            TransformerVariety::DustbinTransformer => { spacebase.map_allotments(|_| LeafCommonStyle::dustbin()) }
        }
    }

    pub fn spacebasearea_transform(&self, coord_system: &CoordinateSystem, spacebase: &SpaceBaseArea<f64,Arc<dyn Transformer>>) -> SpaceBaseArea<f64,LeafCommonStyle> {
        match self {
            TransformerVariety::SimpleTransformer => {
                let items = spacebase.map_allotments(|a| a.into_simple_transformer());
                SimpleTransformerHolder::transform_spacebasearea(coord_system,&items)
            },
            TransformerVariety::DustbinTransformer => { spacebase.map_allotments(|_| LeafCommonStyle::dustbin()) }
        }
    }

    pub fn graph_transform(&self, coord_system: &CoordinateSystem, allot_box: &Arc<dyn Transformer>, values: &[Option<f64>]) -> Vec<Option<f64>> {
        match self {
            TransformerVariety::SimpleTransformer => {
                SimpleTransformerHolder::transform_yy(coord_system,allot_box.into_simple_transformer(),values)
            },
            TransformerVariety::DustbinTransformer => { values.to_vec() }
        }
    }
}
