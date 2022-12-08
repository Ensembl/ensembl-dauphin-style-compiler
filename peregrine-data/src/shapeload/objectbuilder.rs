use anyhow::{ anyhow as err, bail };
use core::f64;
use std::sync::{ Arc, Mutex };
use crate::{Colour, DirectColour, Patina, Pen, Plotter, SpaceBase, LeafRequest, DataRequest, Background, DataResponse, hotspots::zmenupatina::ZMenu};
use owning_ref::ArcRef;
use peregrine_toolkit::{lock, eachorevery::eoestruct::{StructVarGroup, StructTemplate, StructVar, StructPair}};

#[derive(Clone)]
enum ObjectBuilderEntry {
    DirectColour(Arc<DirectColour>),
    Colour(Arc<Colour>),
    Patina(Arc<Patina>),
    ZMenu(Arc<ZMenu>),
    Pen(Arc<Pen>),
    Plotter(Arc<Plotter>),
    LeafRequest(Arc<LeafRequest>),
    SpaceBase(Arc<SpaceBase<f64,()>>),
    DataRequest(Arc<DataRequest>),
    StructGroup(Arc<Mutex<StructVarGroup>>),
    StructTmpl(Arc<StructTemplate>),
    StructVar(Arc<StructVar>),
    StructPair(Arc<StructPair>),
    Background(Arc<Background>),
    DataResponse(Arc<DataResponse>),
}

impl ObjectBuilderEntry {
    fn type_string(&self) -> &str {
        match self {
            ObjectBuilderEntry::DirectColour(_) => "directcolour",
            ObjectBuilderEntry::Colour(_) => "colour",
            ObjectBuilderEntry::Patina(_) => "patina",
            ObjectBuilderEntry::ZMenu(_) => "zmenu",
            ObjectBuilderEntry::Pen(_) => "pen",
            ObjectBuilderEntry::Plotter(_) => "plotter",
            ObjectBuilderEntry::LeafRequest(_) => "allotment",
            ObjectBuilderEntry::SpaceBase(_) => "spacebase",
            ObjectBuilderEntry::DataRequest(_) => "datarequest",
            ObjectBuilderEntry::StructGroup(_) => "eoegroup",
            ObjectBuilderEntry::StructTmpl(_) => "eoetmpl",
            ObjectBuilderEntry::StructVar(_) => "eoevar",
            ObjectBuilderEntry::StructPair(_) => "eoepair",
            ObjectBuilderEntry::Background(_) => "background",
            ObjectBuilderEntry::DataResponse(_) => "data",
        }
    }
}

macro_rules! entry_branch {
    ($value:expr,$branch:tt,$wanted:expr) => {
        if let ObjectBuilderEntry::$branch(x) = $value {
            Ok(ArcRef::new(x))
        } else {
            bail!("expected {} got {}",$wanted,$value.type_string())
        }
    };
}

struct ObjectBuilderData {
    geometry: Vec<ObjectBuilderEntry>
}

fn munge(key: u32) -> u32 { key ^ 0xC85A3 }

impl ObjectBuilderData {
    fn new() -> ObjectBuilderData {
        ObjectBuilderData {
            geometry: vec![]
        }
    }

    fn add(&mut self, entry: ObjectBuilderEntry) -> u32 {
        let out = munge(self.geometry.len() as u32);
        self.geometry.push(entry);
        out
    }

    fn get(&self, id: u32, name: &str) -> anyhow::Result<ObjectBuilderEntry> {
        Ok(self.geometry.get(munge(id) as usize).ok_or(err!("no such {} id",name))?.clone())
    }
}

macro_rules! builder_type {
    ($read:ident,$write:ident,$branch:tt,$typ:ty,$type_name:expr) => {
        pub fn $read(&self, id: u32) -> Result<ArcRef<$typ>,anyhow::Error> {
            entry_branch!(lock!(self.0).get(id,$type_name)?,$branch,$type_name)
        }

        pub fn $write(&self, item: $typ) -> u32 {
            lock!(self.0).add(ObjectBuilderEntry::$branch(Arc::new(item)))
        }
    };
}

#[derive(Clone)]
pub struct ObjectBuilder(Arc<Mutex<ObjectBuilderData>>);

impl ObjectBuilder {
    pub fn new() -> ObjectBuilder {
        ObjectBuilder(Arc::new(Mutex::new(ObjectBuilderData::new())))
    }

    builder_type!(patina,add_patina,Patina,Patina,"patina");
    builder_type!(direct_colour,add_direct_colour,DirectColour,DirectColour,"directcolour");
    builder_type!(colour,add_colour,Colour,Colour,"colour");
    builder_type!(zmenu,add_zmenu,ZMenu,ZMenu,"zmenu");
    builder_type!(pen,add_pen,Pen,Pen,"pen");
    builder_type!(plotter,add_plotter,Plotter,Plotter,"plotter");
    builder_type!(allotment,add_allotment,LeafRequest,LeafRequest,"allotment");
    builder_type!(spacebase,add_spacebase,SpaceBase,SpaceBase<f64,()>,"spacebase");
    builder_type!(request,add_request,DataRequest,DataRequest,"datarequest");
    builder_type!(eoegroup,add_eoegroup,StructGroup,Mutex<StructVarGroup>,"group");
    builder_type!(eoetmpl,add_eoetmpl,StructTmpl,StructTemplate,"template");
    builder_type!(eoevar,add_eoevar,StructVar,StructVar,"variable");
    builder_type!(eoepair,add_eoepair,StructPair,StructPair,"pair");
    builder_type!(background,add_background,Background,Background,"background");
    builder_type!(data,add_data,DataResponse,DataResponse,"data");
}
