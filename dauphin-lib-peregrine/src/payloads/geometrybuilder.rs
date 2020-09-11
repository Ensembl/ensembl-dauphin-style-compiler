use anyhow::{ anyhow as err, bail };
use std::sync::{ Arc, Mutex };
use peregrine_core::{ SeaEndPair, SeaEnd, ShipEnd, lock, Patina, DirectColour, ZMenu, Pen, Plotter };
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
    Plotter(Arc<Plotter>)
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
            GeometryBuilderEntry::Plotter(_) => "plotter"
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

macro_rules! builder_type {
    ($read:ident,$write:ident,$typ:tt,$type_name:expr) => {
        pub fn $read(&self, id: u32) -> anyhow::Result<ArcRef<$typ>> {
            entry_branch!(lock!(self.0).get(id)?,$typ,$type_name)
        }

        pub fn $write(&self, item: $typ) -> u32 {
            lock!(self.0).add(GeometryBuilderEntry::$typ(Arc::new(item)))
        }
    };
}

pub struct GeometryBuilder(Arc<Mutex<GeometryBuilderData>>);

impl GeometryBuilder {
    pub fn new() -> GeometryBuilder {
        GeometryBuilder(Arc::new(Mutex::new(GeometryBuilderData::new())))
    }

    builder_type!(seaendpair,add_seaendpair,SeaEndPair,"seaendpair");
    builder_type!(seaend,add_seaend,SeaEnd,"seaend");
    builder_type!(shipend,add_shipend,ShipEnd,"shipend");
    builder_type!(patina,add_patina,Patina,"patina");
    builder_type!(direct_colour,add_direct_colour,DirectColour,"directcolour");
    builder_type!(zmenu,add_zmenu,ZMenu,"zmenu");
    builder_type!(pen,add_pen,Pen,"pen");
    builder_type!(plotter,add_plotter,Plotter,"plotter");
}
