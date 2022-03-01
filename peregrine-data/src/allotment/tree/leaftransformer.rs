use peregrine_toolkit::puzzle::PuzzleSolution;

use crate::{CoordinateSystem, SpaceBase, SpaceBaseArea, PartialSpaceBase};

use super::allotmentbox::AllotmentBox;

pub fn transform_spacebase(solution: &PuzzleSolution, coord_system: &CoordinateSystem, input: &SpaceBase<f64,AllotmentBox>) -> SpaceBase<f64,AllotmentBox> {
    let mut output = input.clone();
    if coord_system.up_from_bottom() {
        output.update_normal_from_allotment(|n,a| { *n = (a.draw_bottom(solution) as f64) - *n; });
    } else {
        output.update_normal_from_allotment(|n,a| { *n += a.draw_top(solution) as f64; });
    }
    output.update_tangent_from_allotment(|t,a| { *t += a.indent(solution) as f64; });
    output
}

pub fn transform_spacebase2(solution: &PuzzleSolution,coord_system: &CoordinateSystem, input: &SpaceBase<f64,AllotmentBox>) -> SpaceBase<f64,()> {
    let mut output = input.clone();
    if coord_system.up_from_bottom() {
        output.update_normal_from_allotment(|n,a| { *n = (a.draw_bottom(solution) as f64) - *n; });
    } else {
        output.update_normal_from_allotment(|n,a| { *n += a.draw_top(solution) as f64; });
    }
    output.update_tangent_from_allotment(|t,a| { *t += a.indent(solution) as f64; });
    output.fullmap_allotments_results::<_,_,()>(|_| Ok(())).ok().unwrap()
}

pub fn transform_spacebasearea(solution: &PuzzleSolution, coord_system: &CoordinateSystem, input: &SpaceBaseArea<f64,AllotmentBox>) -> SpaceBaseArea<f64,AllotmentBox> {
    let top_left = transform_spacebase(solution,coord_system,input.top_left());
    let bottom_right = transform_spacebase(solution,coord_system,&input.bottom_right());
    SpaceBaseArea::new(PartialSpaceBase::from_spacebase(top_left),PartialSpaceBase::from_spacebase(bottom_right)).unwrap()
}

pub fn transform_spacebasearea2(solution: &PuzzleSolution, coord_system: &CoordinateSystem, input: &SpaceBaseArea<f64,AllotmentBox>) -> SpaceBaseArea<f64,()> {
    let top_left = transform_spacebase2(solution,coord_system,input.top_left());
    let bottom_right = transform_spacebase2(solution,coord_system,&input.bottom_right());
    SpaceBaseArea::new(PartialSpaceBase::from_spacebase(top_left),PartialSpaceBase::from_spacebase(bottom_right)).unwrap()
}

pub fn transform_yy(solution: &PuzzleSolution, coord_system: &CoordinateSystem, allot_box: &AllotmentBox, values: &[Option<f64>]) -> Vec<Option<f64>> {
    if coord_system.up_from_bottom() {
        let offset = allot_box.draw_bottom(solution) as f64;
        values.iter().map(|x| x.map(|y| offset-y)).collect()
    } else {
        let offset = allot_box.draw_top(solution) as f64;
        values.iter().map(|x| x.map(|y| y+offset)).collect()
    }
}
