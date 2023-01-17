use std::{collections::{hash_map::DefaultHasher}, hash::{Hash, Hasher}, sync::Arc, rc::Rc};
use eachorevery::{EachOrEvery, EachOrEveryFilter, eoestruct::{StructTemplate, StructBuilt, struct_select, StructValue}};

use crate::{hotspots::{zmenupatina::ZMenu, hotspots::SpecialClick}, HotspotResult, zmenu_generator, SpaceBasePoint, allotment::leafs::auxleaf::AuxLeaf};
use super::{settingmode::SettingMode};

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub struct DirectColour(pub u8,pub u8,pub u8,pub u8);

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
pub enum AttachmentPoint {
    Left,
    Right
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
    background: EachOrEvery<DirectColour>,
    attachment: AttachmentPoint
}

impl Pen {
    pub fn new(name: &str, size: u32, colours: &EachOrEvery<DirectColour>, background: &EachOrEvery<DirectColour>, attachment: &AttachmentPoint) -> Pen {
        Pen {
            geometry: Arc::new(PenGeometry::new(name,size)),
            colours: colours.clone(),
            background: background.clone(),
            attachment: attachment.clone()
        }
    }

    pub fn geometry(&self) -> &PenGeometry { &self.geometry }
    pub fn colours(&self) -> &EachOrEvery<DirectColour> { &self.colours }
    pub fn background(&self) -> &EachOrEvery<DirectColour> { &self.background }
    pub fn attachment(&self) -> &AttachmentPoint { &self.attachment }

    pub fn filter(&self, filter: &EachOrEveryFilter) -> Pen {
        Pen {
            geometry: self.geometry.clone(),
            colours: self.colours.filter(filter),
            background: self.background.filter(filter),
            attachment: self.attachment.clone()
        }
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
    Special(EachOrEvery<String>),
    Click(Arc<StructTemplate>,Arc<StructTemplate>)
}

fn setting_generator(values: &EachOrEvery<(Vec<String>,SettingMode)>) -> Arc<dyn Fn(usize,Option<(SpaceBasePoint<f64,AuxLeaf>,SpaceBasePoint<f64,AuxLeaf>)>) -> Option<HotspotResult>> {
    let values = Rc::new(values.clone());
    Arc::new(move |index,_| {
        let (path,mode) = values.get(index).unwrap().clone();
        Some(HotspotResult::Setting(path,mode))
    })
}

fn special_generator(values: &EachOrEvery<String>) -> Arc<dyn Fn(usize,Option<(SpaceBasePoint<f64,AuxLeaf>,SpaceBasePoint<f64,AuxLeaf>)>) -> Option<HotspotResult>> {
    let values = Rc::new(values.clone());
    Arc::new(move |index,area| {
        Some(HotspotResult::Special(SpecialClick {
            name: values.get(index).unwrap().to_string(),
            area
        }))
    })
}

pub fn click_generator(variety: &Arc<StructTemplate>, content: &Arc<StructTemplate>) -> Arc<dyn Fn(usize,Option<(SpaceBasePoint<f64,AuxLeaf>,SpaceBasePoint<f64,AuxLeaf>)>) -> Option<HotspotResult>> {
    let variety = variety.clone();
    let content = content.clone();
    Arc::new(move |index,_| {
        let built_content = content.set_index(&[],index).ok()?.build().ok()?;
        let value_content = StructValue::new_expand(&built_content,None).ok()?;
        let built_variety = variety.build().ok()?;
        let value_variety = StructValue::new_expand(&built_variety,None).ok()?;
        Some(HotspotResult::Click(value_variety,value_content))
    })
}

fn ok_or_empty_click<E>(input: Result<StructTemplate,E>) -> StructTemplate {
    input.unwrap_or_else(|_| {
        StructTemplate::new_object(vec![])
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
            },
            HotspotPatina::Click(id,value) => {
                HotspotPatina::Click(id.clone(),Arc::new(ok_or_empty_click(value.filter(&[],filter))))
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
            HotspotPatina::Click(_,value) => { 
                value.compatible(&[],len).unwrap_or(false)
            }
        }
    }

    pub fn generator(&self) -> Arc<dyn Fn(usize,Option<(SpaceBasePoint<f64,AuxLeaf>,SpaceBasePoint<f64,AuxLeaf>)>) -> Option<HotspotResult>> {
        match self {
            HotspotPatina::ZMenu(zmenu,values) => {
                zmenu_generator(&zmenu,values)
            },
            HotspotPatina::Setting(values) => {
                setting_generator(&values)
            },
            HotspotPatina::Special(values) => {
                special_generator(&values)
            },
            HotspotPatina::Click(variety,content) => {
                click_generator(variety,content)
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
