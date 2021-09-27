use std::hash::{ Hash };

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub enum AllotmentDirection {
    Forward,
    Reverse
}

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub enum AllotmentGroup {
    Track,
    Overlay(i64),
    BaseLabel(AllotmentDirection),
    SpaceLabel(AllotmentDirection)
}

impl AllotmentGroup {
    pub(crate) fn base_filter(&self) -> bool {
        match self {
            AllotmentGroup::Track => true,
            AllotmentGroup::Overlay(_) => false,
            AllotmentGroup::BaseLabel(_) => true,
            AllotmentGroup::SpaceLabel(_) => false
        }
    }

    pub fn direction(&self) -> AllotmentDirection {
        match self {
            AllotmentGroup::BaseLabel(d) => d.clone(),
            AllotmentGroup::SpaceLabel(d) => d.clone(),
            _ => AllotmentDirection::Forward
        }
    }
}