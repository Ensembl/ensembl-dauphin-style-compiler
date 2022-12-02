use std::sync::Arc;
use peregrine_toolkit::puzzle::StaticAnswer;
use crate::{CoordinateSystem, allotment::style::leafstyle::LeafStyle};
use super::floating::FloatingLeaf;

#[cfg_attr(any(test,debug_assertions),derive(Debug))]
#[derive(Clone)]
pub struct AnchoredLeaf {
    statics: Arc<LeafStyle>,
    top: f64,
    height: f64,
    indent: f64
}

impl AnchoredLeaf {
    pub fn new(answer_index: &StaticAnswer, floating: &FloatingLeaf) -> AnchoredLeaf {
        AnchoredLeaf {
            statics: floating.statics.clone(),
            top: floating.top.call(answer_index),
            height: floating.max_y_piece.call(answer_index),
            indent: floating.indent.call(answer_index).unwrap_or(0.)
        }
    }

    pub(crate) fn coordinate_system(&self) -> &CoordinateSystem { &self.statics.aux.coord_system }
    pub(crate) fn get_style(&self) -> &LeafStyle { &self.statics }

    #[cfg(any(debug_assertions,test))]
    pub(crate) fn describe(&self) -> String {
        format!("{:?}",self)
    }

    pub(crate) fn top(&self) -> f64 { self.top }
    pub(crate) fn bottom(&self) -> f64 { self.top + self.height }
    pub(crate) fn indent(&self) -> f64 { self.indent }
}
