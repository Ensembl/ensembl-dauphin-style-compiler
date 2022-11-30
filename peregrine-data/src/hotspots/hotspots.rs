use std::{sync::Arc };
use peregrine_toolkit::{ identitynumber, orderable, hashable };
use crate::{ZMenuFixed, SettingMode, SpaceBaseArea, LeafStyle, HotspotPatina, SpaceBasePointRef, SpaceBasePoint};

#[derive(Clone)]
pub struct SpecialClick {
    pub name: String,
    pub area: Option<(SpaceBasePoint<f64,LeafStyle>,SpaceBasePoint<f64,LeafStyle>)>
}

pub enum HotspotResult {
    ZMenu(ZMenuFixed),
    Setting(Vec<String>,SettingMode),
    Special(SpecialClick)
}

impl HotspotResult {
    pub fn get_special(&self) -> Option<SpecialClick> {
        match self {
            HotspotResult::Special(c) => Some(c.clone()),
            _ => None
        }
    }
}

identitynumber!(IDS);

#[derive(Clone)]
pub struct HotspotGroupEntry {
    generator: Arc<dyn Fn(usize,Option<(SpaceBasePoint<f64,LeafStyle>,SpaceBasePoint<f64,LeafStyle>)>) -> HotspotResult>,
    area: SpaceBaseArea<f64,LeafStyle>,
    id: u64
}

hashable!(HotspotGroupEntry,id);
orderable!(HotspotGroupEntry,id);

impl HotspotGroupEntry {
    pub fn new(area: SpaceBaseArea<f64,LeafStyle>, hotspot: &HotspotPatina) -> HotspotGroupEntry {
        HotspotGroupEntry {
            generator: hotspot.generator(),
            area,
            id: IDS.next()
        }
    }

    pub fn area(&self) -> &SpaceBaseArea<f64,LeafStyle> { &self.area }
    pub fn value(&self, index: usize) -> HotspotResult { 
        let top_left = self.area.top_left().get(index).map(|x| x.make());
        let bottom_right = self.area.bottom_right().get(index).map(|x| x.make());
        let position = top_left.zip(bottom_right);
        (self.generator)(index,position)
    }
}

#[derive(Clone)]
pub struct SingleHotspotEntry {
    index: usize,
    order: usize,
    unscaled: HotspotGroupEntry
}

impl SingleHotspotEntry {
    pub fn new(unscaled: &HotspotGroupEntry, index: usize, order: usize) -> SingleHotspotEntry {
        SingleHotspotEntry {
            unscaled: unscaled.clone(),
            index,
            order
        }
    }

    pub fn coordinates(&self) -> Option<(SpaceBasePointRef<f64,LeafStyle>,SpaceBasePointRef<f64,LeafStyle>)> {
        self.unscaled.area().iter().nth(self.index)
    }

    pub fn value(&self) -> HotspotResult {
        self.unscaled.value(self.index)
    }
}

impl PartialEq for SingleHotspotEntry {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.order == other.order && self.unscaled == other.unscaled
    }
}

impl Eq for SingleHotspotEntry {}

impl PartialOrd for SingleHotspotEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.order.partial_cmp(&other.order)
    }
}

impl Ord for SingleHotspotEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.order.cmp(&other.order)
    }
}
