use std::sync::Arc;
use crate::{CoordinateSystem, SpaceBase, SpaceBaseArea, PartialSpaceBase, allotment::style::style::LeafCommonStyle};

use super::{transformertraits::{SpaceBaseTransformer, GraphTransformer}, simple::SimpleTransformerHolder};

#[derive(Clone)]
#[derive(Hash,PartialEq,Eq)]
pub enum TransformerVariety {
    SimpleTransformer
}

pub trait Transformer {
    fn choose_variety(&self) -> (TransformerVariety,CoordinateSystem);
    fn into_simple_transformer(&self) -> SimpleTransformerHolder { panic!(); }
    fn get_style(&self) -> &LeafCommonStyle;

    #[cfg(test)]
    fn describe(&self) -> String;
}

impl TransformerVariety {
    pub fn spacebase_transform(&self, coord_system: &CoordinateSystem, spacebase: &SpaceBase<f64,Arc<dyn Transformer>>) -> SpaceBase<f64,LeafCommonStyle> {
        match self {
            TransformerVariety::SimpleTransformer => {
                let items = spacebase.map_allotments(|a| a.into_simple_transformer());
                SimpleTransformerHolder::transform_spacebase(coord_system,&items)
            }
        }
    }

    pub fn spacebasearea_transform(&self, coord_system: &CoordinateSystem, spacebase: &SpaceBaseArea<f64,Arc<dyn Transformer>>) -> SpaceBaseArea<f64,LeafCommonStyle> {
        match self {
            TransformerVariety::SimpleTransformer => {
                let items = spacebase.map_allotments(|a| a.into_simple_transformer());
                SimpleTransformerHolder::transform_spacebasearea(coord_system,&items)
            }
        }
    }

    pub fn graph_transform(&self, coord_system: &CoordinateSystem, allot_box: &Arc<dyn Transformer>, values: &[Option<f64>]) -> Vec<Option<f64>> {
        match self {
            TransformerVariety::SimpleTransformer => {
                SimpleTransformerHolder::transform_yy(coord_system,&allot_box.into_simple_transformer(),values)
            }
        }
    }
}

fn spacebase_transform(spacebase: SpaceBase<f64,Arc<dyn Transformer>>) -> Vec<SpaceBase<f64,LeafCommonStyle>> {
    let mut out = vec![];
    for ((variety,coord_system),filter) in spacebase.demerge_by_allotment(|x| x.choose_variety()).drain(..) {
        let items = spacebase.filter(&filter);
        out.push(variety.spacebase_transform(&coord_system,&items));
    }
    out
}

fn spacebasearea_transform(spacebase: SpaceBaseArea<f64,Arc<dyn Transformer>>) -> Vec<SpaceBaseArea<f64,LeafCommonStyle>> {
    let mut out = vec![];
    for ((variety,coord_system),filter) in spacebase.demerge_by_allotment(|x| x.choose_variety()).drain(..) {
        let items = spacebase.filter(&filter);
        out.push(variety.spacebasearea_transform(&coord_system,&items));
    }
    out
}

fn graph_transform(allot_box: &Arc<dyn Transformer>, values: &[Option<f64>]) -> Vec<Option<f64>> {
    let (variety,coord_system) = allot_box.choose_variety();
    variety.graph_transform(&coord_system,allot_box,values)
}
