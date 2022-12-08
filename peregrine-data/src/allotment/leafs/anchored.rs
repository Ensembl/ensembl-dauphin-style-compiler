use std::sync::Arc;
use peregrine_toolkit::puzzle::StaticAnswer;
use crate::{CoordinateSystem, allotment::style::leafstyle::LeafStyle};
use super::floating::FloatingLeaf;
use crate::{SpaceBase, SpaceBaseArea, PartialSpaceBase, AuxLeaf};

#[cfg_attr(any(test,debug_assertions),derive(Debug))]
#[derive(Clone)]
pub(crate) struct AnchoredLeaf {
    statics: Arc<LeafStyle>,
    top: f64,
    height: f64,
    indent: f64
}

impl AnchoredLeaf {
    pub(crate) fn new(answer_index: &StaticAnswer, floating: &FloatingLeaf) -> AnchoredLeaf {
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

impl SpaceBaseArea<f64,AnchoredLeaf> {
    pub fn spacebasearea_transform(&self, coord_system: &CoordinateSystem) -> SpaceBaseArea<f64,AuxLeaf> {
        let top_left = self.top_left().spacebase_transform(coord_system);
        let bottom_right = self.bottom_right().spacebase_transform(coord_system);
        SpaceBaseArea::new(PartialSpaceBase::from_spacebase(top_left),PartialSpaceBase::from_spacebase(bottom_right)).unwrap()
    }
}

impl SpaceBase<f64,AnchoredLeaf> {
    pub fn spacebase_transform(&self, coord_system: &CoordinateSystem) -> SpaceBase<f64,AuxLeaf> {
        let mut output = self.clone();
        if coord_system.up_from_bottom() {
            output.update_normal_from_allotment(|n,a| { 
                *n = a.bottom() - *n;
            });
        } else {
            output.update_normal_from_allotment(|n,a| { 
                *n += a.top();
            });
        }
        output.update_tangent_from_allotment(|t,a| { 
            *t += a.indent();
        });
        output.into_new_allotment(|x| { x.get_style().aux.clone() })
    }
}

pub(crate) fn yy_transform(coord_system: &CoordinateSystem, allot_box: &AnchoredLeaf, values: &[Option<f64>]) -> Vec<Option<f64>> {
    if coord_system.up_from_bottom() {
        let offset = allot_box.bottom();
        values.iter().map(|x| x.map(|y| offset-y)).collect()
    } else {
        let offset = allot_box.top();
        values.iter().map(|x| x.map(|y| y+offset)).collect()
    }
}
