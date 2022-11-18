use std::{sync::Arc };
use peregrine_toolkit::{ identitynumber, orderable, hashable };
use crate::{ZMenuFixed, SettingMode, SpaceBaseArea, LeafStyle, HotspotPatina, SpaceBasePointRef};

pub enum HotspotResult {
    ZMenu(ZMenuFixed),
    Setting(Vec<String>,SettingMode),
    Special(String)
}

identitynumber!(IDS);

#[derive(Clone)]
pub struct HotspotGroupEntry {
    generator: Arc<dyn Fn(usize) -> HotspotResult>,
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
    pub fn value(&self, index: usize) -> HotspotResult { (self.generator)(index) }
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
