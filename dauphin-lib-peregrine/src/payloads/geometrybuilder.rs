use anyhow::{ anyhow as err, bail };
use std::sync::{ Arc, Mutex };
use peregrine_core::{ SeaEndPair, SeaEnd, ShipEnd, lock };
use owning_ref::ArcRef;

#[derive(Clone)]
enum GeometryBuilderEntry {
    SeaEndPair(Arc<SeaEndPair>),
    SeaEnd(Arc<SeaEnd>),
    ShipEnd(Arc<ShipEnd>)
}

impl GeometryBuilderEntry {
    fn type_string(&self) -> &str {
        match self {
            GeometryBuilderEntry::SeaEndPair(_) => "seaendpair",
            GeometryBuilderEntry::SeaEnd(_) => "seaend",
            GeometryBuilderEntry::ShipEnd(_) => "shipend",
        }
    }
}

macro_rules! entry_branch {
    ($value:expr,$branch:tt,$wanted:expr) => {
        if let GeometryBuilderEntry::$branch(x) = $value {
            Ok(ArcRef::new(x))
        } else {
            bail!("expected {} got {}",$wanted,$value.type_string())
        }
    };
}

struct GeometryBuilderData {
    geometry: Vec<GeometryBuilderEntry>
}

fn munge(key: u32) -> u32 { key ^ 0xC85A3 }

impl GeometryBuilderData {
    fn new() -> GeometryBuilderData {
        GeometryBuilderData {
            geometry: vec![]
        }
    }

    fn add(&mut self, entry: GeometryBuilderEntry) -> u32 {
        let out = munge(self.geometry.len() as u32);
        self.geometry.push(entry);
        out
    }

    fn get(&self, id: u32) -> anyhow::Result<GeometryBuilderEntry> {
        Ok(self.geometry.get(munge(id) as usize).ok_or(err!("bad panel id"))?.clone())
    }
}

pub struct GeometryBuilder(Arc<Mutex<GeometryBuilderData>>);

impl GeometryBuilder {
    pub fn new() -> GeometryBuilder {
        GeometryBuilder(Arc::new(Mutex::new(GeometryBuilderData::new())))
    }

    pub fn seaendpair(&self, id: u32) -> anyhow::Result<ArcRef<SeaEndPair>> {
        entry_branch!(lock!(self.0).get(id)?,SeaEndPair,"seaendpair")
    }

    pub fn seaend(&self, id: u32) -> anyhow::Result<ArcRef<SeaEnd>> {
        entry_branch!(lock!(self.0).get(id)?,SeaEnd,"seaend")
    }

    pub fn shipend(&self, id: u32) -> anyhow::Result<ArcRef<ShipEnd>> {
        entry_branch!(lock!(self.0).get(id)?,ShipEnd,"shipend")
    }

    pub fn add_seaendpair(&self, item: SeaEndPair) -> u32 {
        lock!(self.0).add(GeometryBuilderEntry::SeaEndPair(Arc::new(item)))
    }

    pub fn add_seaend(&self, item: SeaEnd) -> u32 {
        lock!(self.0).add(GeometryBuilderEntry::SeaEnd(Arc::new(item)))
    }

    pub fn add_shipend(&self, item: ShipEnd) -> u32 {
        lock!(self.0).add(GeometryBuilderEntry::ShipEnd(Arc::new(item)))
    }
}
