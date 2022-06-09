use std::sync::Arc;

use crate::{CoordinateSystem, SpaceBase, allotment::style::style::LeafStyle};

use super::{transformertraits::{SpaceBaseTransformer, GraphTransformer}};

pub trait SimpleTransformer {
    fn top(&self) -> f64;
    fn bottom(&self) -> f64;
    fn indent(&self) -> f64;
    fn as_simple_transformer(&self) -> &dyn SimpleTransformer;
    fn get_style(&self) -> &LeafStyle;

}

#[derive(Clone)]
pub struct SimpleTransformerHolder(pub Arc<dyn SimpleTransformer>);

impl SpaceBaseTransformer for SimpleTransformerHolder {
    type X = SimpleTransformerHolder;

    fn transform_spacebase(coord_system: &CoordinateSystem, input: &SpaceBase<f64,Option<SimpleTransformerHolder>>) -> SpaceBase<f64,LeafStyle> {
        let mut output = input.clone();
        if coord_system.up_from_bottom() {
            output.update_normal_from_allotment(|n,a| { 
                *n = (a.as_ref().map(|x| x.0.bottom() as f64)).unwrap_or(0.) - *n;
            });
        } else {
            output.update_normal_from_allotment(|n,a| { 
                *n += a.as_ref().map(|x| x.0.top()).unwrap_or(0.) as f64; 
            });
        }
        output.update_tangent_from_allotment(|t,a| { 
            *t += a.as_ref().map(|x| x.0.indent() as f64).unwrap_or(0.)
        });
        output.into_new_allotment(|x| {
            x.as_ref().map(|x| x.0.get_style().clone()).unwrap_or_else(|| LeafStyle::dustbin())
        })
    }
}

impl GraphTransformer for SimpleTransformerHolder {
    type X = SimpleTransformerHolder;

    fn transform_yy(coord_system: &CoordinateSystem, allot_box: Option<Self::X>, values: &[Option<f64>]) -> Vec<Option<f64>> {
        if coord_system.up_from_bottom() {
            let offset = (allot_box.map(|x| x.0.bottom() as f64)).unwrap_or(0.);
            values.iter().map(|x| x.map(|y| offset-y)).collect()
        } else {
            let offset = (allot_box.map(|x| x.0.top() as f64)).unwrap_or(0.);
            values.iter().map(|x| x.map(|y| y+offset)).collect()
        }
    }
}
