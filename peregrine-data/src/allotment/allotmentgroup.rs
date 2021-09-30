use std::hash::{ Hash };

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub enum AllotmentDirection {
    Forward,
    Reverse
}

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub enum AllotmentGroup {
    Track,
    Overlay,
    BaseLabel(AllotmentDirection),
    SpaceLabel(AllotmentDirection)
}

impl AllotmentGroup {
    pub fn direction(&self) -> AllotmentDirection {
        match self {
            AllotmentGroup::BaseLabel(d) => d.clone(),
            AllotmentGroup::SpaceLabel(d) => d.clone(),
            _ => AllotmentDirection::Forward
        }
    }
}
