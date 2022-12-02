use crate::CoordinateSystem;

#[cfg_attr(any(test,debug_assertions),derive(Debug))]
#[derive(Clone)]
pub struct AuxLeaf {
    pub coord_system: CoordinateSystem,
    pub depth: i8,
}
