use std::{collections::{hash_map::DefaultHasher}, hash::{Hash, Hasher}, sync::Arc};
use peregrine_toolkit::eachorevery::{EachOrEveryFilter, EachOrEvery, eoestruct::StructTemplate};

use super::zmenu::ZMenu;

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub struct DirectColour(pub u8,pub u8,pub u8,pub u8);

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
pub enum AttachmentPoint {
    Left,
    Right
}

#[derive(Clone,Hash)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct Background {
    pub colour: DirectColour,
    pub round: bool
}

impl Background {
    pub fn none() -> Background {
        Background {
            colour: DirectColour(255,255,255,0),
            round: false
        }
    }
}

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub struct PenGeometry {
    name: String,
    size: u32,
    hash: u64
}

impl PenGeometry {
    fn new(name: &str, size:u32) -> PenGeometry {
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        size.hash(&mut hasher);
        let hash = hasher.finish();
        PenGeometry {
            name: name.to_string(),
            size,
            hash
        }
    }

    pub fn name(&self) -> &str { &self.name }
    pub fn size_in_webgl(&self) -> f64 { self.size as f64 }
    pub fn group_hash(&self) -> u64 { self.hash }
}

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
pub struct Pen {
    geometry: Arc<PenGeometry>,
    colours: EachOrEvery<DirectColour>,
    background: Option<Background>,
    attachment: AttachmentPoint
}

impl Pen {
    fn new_real(geometry: &Arc<PenGeometry>, colours: &EachOrEvery<DirectColour>, background: &Option<Background>, attachment: &AttachmentPoint) -> Pen {
        Pen {
            geometry: geometry.clone(),
            colours: colours.clone(),
            background: background.clone(),
            attachment: attachment.clone()
        }
    }

    pub fn new(name: &str, size: u32, colours: &[DirectColour], background: &Option<Background>, attachment: &AttachmentPoint) -> Pen {
        let colours = if colours.len() == 1 {
            EachOrEvery::every(colours[0].clone())
        } else {
            EachOrEvery::each(colours.to_vec())
        };
        Pen::new_real(&Arc::new(PenGeometry::new(name,size)), &colours.index(|x| x.clone()),background,attachment)
    }

    pub fn geometry(&self) -> &PenGeometry { &self.geometry }
    pub fn colours(&self) -> &EachOrEvery<DirectColour> { &self.colours }
    pub fn background(&self) -> &Option<Background> { &self.background }
    pub fn attachment(&self) -> &AttachmentPoint { &self.attachment }

    pub fn filter(&self, filter: &EachOrEveryFilter) -> Pen {
        Pen::new_real(&self.geometry,&self.colours.filter(filter),&self.background,&self.attachment)
    }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct Plotter(pub f64, pub DirectColour);

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub enum Colour {
    Direct(DirectColour),
    Spot(DirectColour),
    Stripe(DirectColour,DirectColour,(u32,u32),f64),
    Bar(DirectColour,DirectColour,(u32,u32),f64)
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub enum DrawnType {
    Fill,
    Stroke(f64)
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub enum SettingMode {
    Set(bool),
    Member(String,bool)
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub enum Hotspot {
    ZMenu(ZMenu,Vec<(String,EachOrEvery<String>)>),
    Switch(EachOrEvery<(Vec<String>,bool)>),
    Setting(EachOrEvery<(Vec<String>,SettingMode)>)
}

impl Hotspot {
    fn filter(&self, filter: &EachOrEveryFilter) -> Hotspot {
        match self {
            Hotspot::ZMenu(zmenu,values) => {
                let mut out = Vec::with_capacity(values.len());
                for (k,v) in values {
                    out.push((k.to_string(),v.filter(filter)));
                }
                Hotspot::ZMenu(zmenu.clone(),out)          
            },
            Hotspot::Switch(values) => {
                Hotspot::Switch(values.filter(filter))
            },
            Hotspot::Setting(values) => {
                Hotspot::Setting(values.filter(filter))
            }
        }
    }

    fn compatible(&self, len: usize) -> bool {
        match self {
            Hotspot::ZMenu(_,values) => {
                for (_,value) in values.iter() {
                    if !value.compatible(len) { return false; }
                }
                true
            },
            Hotspot::Switch(value) => { value.compatible(len) },
            Hotspot::Setting(value) => { value.compatible(len) }
        }
    }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub enum Patina {
    Drawn(DrawnType,EachOrEvery<Colour>),
    Hotspot(Hotspot),
    Metadata(String,EachOrEvery<(String,StructTemplate)>)
}

impl Patina {
    pub fn filter(&self, filter: &EachOrEveryFilter) -> Patina {
        match self {
            Patina::Drawn(drawn_type,colours) => Patina::Drawn(drawn_type.clone(),colours.filter(filter)),
            Patina::Hotspot(hotspot) => Patina::Hotspot(hotspot.filter(filter)),
            Patina::Metadata(key,values) => Patina::Metadata(key.clone(),values.filter(filter))
        }
    }

    pub fn compatible(&self, len: usize) -> bool {
        match self {
            Patina::Drawn(_,x) => x.compatible(len),
            Patina::Hotspot(hotspot) => { hotspot.compatible(len) },
            Patina::Metadata(_,values) => { values.compatible(len) }
        }
    }
}
