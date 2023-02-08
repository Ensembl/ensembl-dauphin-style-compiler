use std::{sync::Arc };
use eachorevery::{eoestruct::{StructValue, StructBuilt}, EachOrEvery};
use peregrine_toolkit::{ identitynumber, orderable, hashable };
use crate::{SpaceBaseArea, HotspotPatina, SpaceBasePointRef, SpaceBasePoint, allotment::leafs::auxleaf::AuxLeaf};

#[derive(Clone)]
pub struct SpecialClick {
    pub name: String,
    pub area: Option<(SpaceBasePoint<f64,AuxLeaf>,SpaceBasePoint<f64,AuxLeaf>)>,
    pub run: Option<f64>
}

pub enum HotspotResultVariety {
    Setting(Vec<String>,String,StructBuilt),
    Special(SpecialClick),
    Click(StructValue,StructValue)
}

pub struct HotspotResult {
    pub variety: HotspotResultVariety,
    pub depth: i8
}

impl HotspotResult {
    pub fn get_special(&self) -> Option<SpecialClick> {
        match &self.variety {
            HotspotResultVariety::Special(c) => Some(c.clone()),
            _ => None
        }
    }
}

identitynumber!(IDS);

#[derive(Clone)]
pub struct HotspotGroupEntry {
    generator: Arc<dyn Fn(usize,Option<(SpaceBasePoint<f64,AuxLeaf>,SpaceBasePoint<f64,AuxLeaf>)>,Option<f64>) -> Option<HotspotResultVariety>>,
    hover: bool,
    area: SpaceBaseArea<f64,AuxLeaf>,
    run: Option<EachOrEvery<f64>>,
    depth: EachOrEvery<i8>,
    id: u64
}

hashable!(HotspotGroupEntry,id);
orderable!(HotspotGroupEntry,id);

impl HotspotGroupEntry {
    pub fn new(area: SpaceBaseArea<f64,AuxLeaf>, run: Option<EachOrEvery<f64>>, hotspot: &HotspotPatina, depth: EachOrEvery<i8>, hover: bool) -> HotspotGroupEntry {
        HotspotGroupEntry {
            generator: hotspot.generator(),
            hover, area, run, depth,
            id: IDS.next()
        }
    }

    pub fn area(&self) -> &SpaceBaseArea<f64,AuxLeaf> { &self.area }
    pub fn run(&self) -> &Option<EachOrEvery<f64>> { &self.run }
    pub fn value(&self, index: usize) -> Option<HotspotResult> { 
        let top_left = self.area.top_left().get(index).map(|x| x.make());
        let bottom_right = self.area.bottom_right().get(index).map(|x| x.make());
        let run = self.run.as_ref().and_then(|x| x.get(index).cloned());
        let position = top_left.zip(bottom_right);
        let depth = *self.depth.get(index).unwrap_or(&0);
        (self.generator)(index,position,run).map(|variety| {
            HotspotResult { variety, depth }
        })
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

    pub fn is_hover(&self) -> bool { self.unscaled.hover }

    pub fn coordinates(&self) -> (Option<(SpaceBasePointRef<f64,AuxLeaf>,SpaceBasePointRef<f64,AuxLeaf>)>,Option<f64>) {
        (
            self.unscaled.area().iter().nth(self.index),
            self.unscaled.run().as_ref().and_then(|x| x.get(self.index).cloned())
        )
    }

    pub fn value(&self) -> Option<HotspotResult> {
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
