use std::sync::Arc;

use crate::{CoordinateSystem, SpaceBase};

use super::{transformertraits::{SpaceBaseTransformer, GraphTransformer}};

pub trait SimpleTransformer {
    fn top(&self) -> f64;
    fn bottom(&self) -> f64;
    fn indent(&self) -> f64;
    fn as_simple_transformer(&self) -> &dyn SimpleTransformer;
}

#[derive(Clone)]
pub struct SimpleTransformerHolder(pub Arc<dyn SimpleTransformer>);

impl SpaceBaseTransformer for SimpleTransformerHolder {
    type X = SimpleTransformerHolder;

    fn transform_spacebase(coord_system: &CoordinateSystem, input: &SpaceBase<f64,SimpleTransformerHolder>) -> SpaceBase<f64,()> {
        let mut output = input.clone();
        if coord_system.up_from_bottom() {
            output.update_normal_from_allotment(|n,a| { *n = (a.0.bottom() as f64) - *n; });
        } else {
            output.update_normal_from_allotment(|n,a| { *n += a.0.top() as f64; });
        }
        output.update_tangent_from_allotment(|t,a| { *t += a.0.indent() as f64; });
        output.fullmap_allotments_results::<_,_,()>(|_| Ok(())).ok().unwrap()    
    }
}

impl GraphTransformer for SimpleTransformerHolder {
    type X = SimpleTransformerHolder;

    fn transform_yy(coord_system: &CoordinateSystem, allot_box: &Self::X, values: &[Option<f64>]) -> Vec<Option<f64>> {
        if coord_system.up_from_bottom() {
            let offset = allot_box.0.bottom() as f64;
            values.iter().map(|x| x.map(|y| offset-y)).collect()
        } else {
            let offset = allot_box.0.top() as f64;
            values.iter().map(|x| x.map(|y| y+offset)).collect()
        }
    }
}
