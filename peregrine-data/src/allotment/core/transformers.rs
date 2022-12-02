use crate::{SpaceBase, allotment::{style::leafstyle::{LeafStyle}, boxes::leaf::{AnchoredLeaf, AuxLeaf}}, CoordinateSystem, SpaceBaseArea, PartialSpaceBase};

pub(crate) trait SimpleTransformer {
    fn top(&self) -> f64;
    fn bottom(&self) -> f64;
    fn indent(&self) -> f64;
    fn as_simple_transformer(&self) -> &dyn SimpleTransformer;
    fn get_style(&self) -> &LeafStyle;

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

pub fn yy_transform(coord_system: &CoordinateSystem, allot_box: &AnchoredLeaf, values: &[Option<f64>]) -> Vec<Option<f64>> {
    if coord_system.up_from_bottom() {
        let offset = allot_box.bottom();
        values.iter().map(|x| x.map(|y| offset-y)).collect()
    } else {
        let offset = allot_box.top();
        values.iter().map(|x| x.map(|y| y+offset)).collect()
    }
}
