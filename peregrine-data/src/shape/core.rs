use std::{collections::{hash_map::DefaultHasher}, hash::{Hash, Hasher}, sync::Arc, rc::Rc};
use peregrine_toolkit::eachorevery::{EachOrEveryFilter, EachOrEvery, eoestruct::{StructTemplate}};
use crate::{hotspots::{zmenupatina::ZMenu, hotspots::SpecialClick}, HotspotResult, zmenu_generator, SpaceBasePoint, allotment::boxes::leaf::AuxLeaf};
use super::{settingmode::SettingMode};

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
pub enum HotspotPatina {
    ZMenu(ZMenu,Vec<(String,EachOrEvery<String>)>),
    Setting(EachOrEvery<(Vec<String>,SettingMode)>),
    Special(EachOrEvery<String>)
}

fn setting_generator(values: &EachOrEvery<(Vec<String>,SettingMode)>) -> Arc<dyn Fn(usize,Option<(SpaceBasePoint<f64,AuxLeaf>,SpaceBasePoint<f64,AuxLeaf>)>) -> HotspotResult> {
    let values = Rc::new(values.clone());
    Arc::new(move |index,_| {
        let (path,mode) = values.get(index).unwrap().clone();
        HotspotResult::Setting(path,mode)
    })
}

fn special_generator(values: &EachOrEvery<String>) -> Arc<dyn Fn(usize,Option<(SpaceBasePoint<f64,AuxLeaf>,SpaceBasePoint<f64,AuxLeaf>)>) -> HotspotResult> {
    let values = Rc::new(values.clone());
    Arc::new(move |index,area| {
        HotspotResult::Special(SpecialClick {
            name: values.get(index).unwrap().to_string(),
            area
        })
    })
}

impl HotspotPatina {
    fn filter(&self, filter: &EachOrEveryFilter) -> HotspotPatina {
        match self {
            HotspotPatina::ZMenu(zmenu,values) => {
                let mut out = Vec::with_capacity(values.len());
                for (k,v) in values {
                    out.push((k.to_string(),v.filter(filter)));
                }
                HotspotPatina::ZMenu(zmenu.clone(),out)          
            },
            HotspotPatina::Setting(values) => {
                HotspotPatina::Setting(values.filter(filter))
            },
            HotspotPatina::Special(value) => {
                HotspotPatina::Special(value.filter(filter))
            }
        }
    }

    fn compatible(&self, len: usize) -> bool {
        match self {
            HotspotPatina::ZMenu(_,values) => {
                for (_,value) in values.iter() {
                    if !value.compatible(len) { return false; }
                }
                true
            },
            HotspotPatina::Setting(value) => { value.compatible(len) },
            HotspotPatina::Special(value) => { value.compatible(len) },
        }
    }

    pub fn generator(&self) -> Arc<dyn Fn(usize,Option<(SpaceBasePoint<f64,AuxLeaf>,SpaceBasePoint<f64,AuxLeaf>)>) -> HotspotResult> {
        match self {
            HotspotPatina::ZMenu(zmenu,values) => {
                zmenu_generator(&zmenu,values)
            },
            HotspotPatina::Setting(values) => {
                setting_generator(&values)
            },
            HotspotPatina::Special(values) => {
                special_generator(&values)
            }
        }
    }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub enum Patina {
    Drawn(DrawnType,EachOrEvery<Colour>),
    Hotspot(HotspotPatina),
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
