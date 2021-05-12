use anyhow::{ anyhow as err, bail };
use std::sync::{ Arc, Mutex };
use peregrine_data::{ SeaEndPair, SeaEnd, ShipEnd, lock, Patina, DirectColour, ZMenu, Pen, Plotter, AllotmentHandle, SpaceBase };
use owning_ref::ArcRef;

#[derive(Clone)]
enum GeometryBuilderEntry {
    SeaEndPair(Arc<SeaEndPair>),
    SeaEnd(Arc<SeaEnd>),
    ShipEnd(Arc<ShipEnd>),
    DirectColour(Arc<DirectColour>),
    Patina(Arc<Patina>),
    ZMenu(Arc<ZMenu>),
    Pen(Arc<Pen>),
    Plotter(Arc<Plotter>),
    Allotment(Arc<AllotmentHandle>),
    SpaceBase(Arc<SpaceBase<AllotmentHandle>>),
}

impl GeometryBuilderEntry {
    fn type_string(&self) -> &str {
        match self {
            GeometryBuilderEntry::SeaEndPair(_) => "seaendpair",
            GeometryBuilderEntry::SeaEnd(_) => "seaend",
            GeometryBuilderEntry::ShipEnd(_) => "shipend",
            GeometryBuilderEntry::DirectColour(_) => "directcolour",
            GeometryBuilderEntry::Patina(_) => "patina",
            GeometryBuilderEntry::ZMenu(_) => "zmenu",
            GeometryBuilderEntry::Pen(_) => "pen",
            GeometryBuilderEntry::Plotter(_) => "plotter",
            GeometryBuilderEntry::Allotment(_) => "allotment",
            GeometryBuilderEntry::SpaceBase(_) => "spacebase",
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
        Ok(self.geometry.get(munge(id) as usize).ok_or(err!("bad lane id"))?.clone())
    }
}

macro_rules! builder_type {
    ($read:ident,$write:ident,$branch:tt,$typ:ty,$type_name:expr) => {
        pub fn $read(&self, id: u32) -> anyhow::Result<ArcRef<$typ>> {
            entry_branch!(lock!(self.0).get(id)?,$branch,$type_name)
        }

        pub fn $write(&self, item: $typ) -> u32 {
            lock!(self.0).add(GeometryBuilderEntry::$branch(Arc::new(item)))
        }
    };
}

pub struct GeometryBuilder(Arc<Mutex<GeometryBuilderData>>);

impl GeometryBuilder {
    pub fn new() -> GeometryBuilder {
        GeometryBuilder(Arc::new(Mutex::new(GeometryBuilderData::new())))
    }

    builder_type!(seaendpair,add_seaendpair,SeaEndPair,SeaEndPair,"seaendpair");
    builder_type!(seaend,add_seaend,SeaEnd,SeaEnd,"seaend");
    builder_type!(shipend,add_shipend,ShipEnd,ShipEnd,"shipend");
    builder_type!(patina,add_patina,Patina,Patina,"patina");
    builder_type!(direct_colour,add_direct_colour,DirectColour,DirectColour,"directcolour");
    builder_type!(zmenu,add_zmenu,ZMenu,ZMenu,"zmenu");
    builder_type!(pen,add_pen,Pen,Pen,"pen");
    builder_type!(plotter,add_plotter,Plotter,Plotter,"plotter");
    builder_type!(allotment,add_allotment,Allotment,AllotmentHandle,"allotment");
    builder_type!(spacebase,add_spacebase,SpaceBase,SpaceBase<AllotmentHandle>,"spacebase");
}
