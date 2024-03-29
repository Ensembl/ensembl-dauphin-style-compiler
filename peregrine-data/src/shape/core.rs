use std::{collections::{hash_map::DefaultHasher}, hash::{Hash, Hasher}, sync::Arc, rc::Rc};
use eachorevery::{EachOrEvery, EachOrEveryFilter, eoestruct::{StructTemplate, StructValue, StructBuilt}};
use crate::{hotspots::{hotspots::{SpecialClick, HotspotResultVariety}}, SpaceBasePoint, allotment::leafs::auxleaf::AuxLeaf};

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

    pub fn to_font(&self, bitmap_multiplier: f64) -> String {
        let mut name = self.name().trim();
        let mut prefix = String::new();
        loop {
            if name.len() == 0 { break; }
            if name.starts_with("bold ") {
                prefix.push_str("bold");
                name = &name["bold".len()..];
            } else if name.starts_with("italic") {
                prefix.push_str("italic ");
                name = &name["italic".len()..];                
            } else {
                let mut number = String::new();
                let mut chars = name.chars();
                while let Some(digit) = chars.next().filter(|x| x.is_digit(10)) {
                    number.push(digit);
                }
                if number.len() > 0 {
                    prefix.push_str(&number);
                    prefix.push(' ');
                    name = &name[number.len()..];
                } else {
                    break;
                }
            }
        }
        format!("{} {}px {}",prefix,(self.size_in_webgl() * bitmap_multiplier).round(),name)
    }    
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
    Setting(Vec<String>,EachOrEvery<(String,StructBuilt)>),
    Special(EachOrEvery<String>),
    Click(Arc<StructTemplate>,Arc<StructTemplate>)
}

fn setting_generator(switch: &Vec<String>, values: &EachOrEvery<(String,StructBuilt)>) -> Arc<dyn Fn(usize,Option<(SpaceBasePoint<f64,AuxLeaf>,SpaceBasePoint<f64,AuxLeaf>)>,Option<f64>) -> Option<HotspotResultVariety>> {
    let values = Rc::new(values.clone());
    let switch = switch.to_vec();
    Arc::new(move |index,_,_| {
        let (key,value) = values.get(index).unwrap().clone();
        Some(HotspotResultVariety::Setting(switch.clone(),key,value))
    })
}

fn special_generator(values: &EachOrEvery<String>) -> Arc<dyn Fn(usize,Option<(SpaceBasePoint<f64,AuxLeaf>,SpaceBasePoint<f64,AuxLeaf>)>,Option<f64>) -> Option<HotspotResultVariety>> {
    let values = Rc::new(values.clone());
    Arc::new(move |index,area,run| {
        Some(HotspotResultVariety::Special(SpecialClick {
                name: values.get(index).unwrap().to_string(),
                area,
                run
        }))
    })
}

pub fn click_generator(variety: &Arc<StructTemplate>, content: &Arc<StructTemplate>) -> Arc<dyn Fn(usize,Option<(SpaceBasePoint<f64,AuxLeaf>,SpaceBasePoint<f64,AuxLeaf>)>,Option<f64>) -> Option<HotspotResultVariety>> {
    let variety = variety.clone();
    let content = content.clone();
    Arc::new(move |index,_,_| {
        let built_content = content.set_index(&[],index).ok()?.build().ok()?;
        let value_content = StructValue::new_expand(&built_content,None).ok()?;
        let value_content = match value_content {
            StructValue::Array(a) => a.get(0).cloned(),
            _ => None
        };
        let value_content = value_content.unwrap_or(StructValue::new_null());
        let built_variety = variety.build().ok()?;
        let value_variety = StructValue::new_expand(&built_variety,None).ok()?;
        if value_variety == StructValue::new_null() {
            Some(HotspotResultVariety::Nothing)
        } else {
            Some(HotspotResultVariety::Click(value_variety,value_content))
        }
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
            HotspotPatina::Setting(key,values) => {
                HotspotPatina::Setting(key.clone(),values.filter(filter))
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
            HotspotPatina::Setting(_,values) => { values.compatible(len) },
            HotspotPatina::Special(value) => { value.compatible(len) },
            HotspotPatina::Click(_,value) => { 
                value.compatible(&[],len).unwrap_or(false)
            }
        }
    }

    pub fn generator(&self) -> Arc<dyn Fn(usize,Option<(SpaceBasePoint<f64,AuxLeaf>,SpaceBasePoint<f64,AuxLeaf>)>,Option<f64>) -> Option<HotspotResultVariety>> {
        match self {
            HotspotPatina::Setting(key,values) => {
                setting_generator(key,&values)
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
    Hotspot(HotspotPatina,bool),
    Metadata(String,EachOrEvery<(String,StructValue)>)
}

impl Patina {
    pub fn filter(&self, filter: &EachOrEveryFilter) -> Patina {
        match self {
            Patina::Drawn(drawn_type,colours) => Patina::Drawn(drawn_type.clone(),colours.filter(filter)),
            Patina::Hotspot(hotspot,hover) => Patina::Hotspot(hotspot.filter(filter),*hover),
            Patina::Metadata(key,values) => Patina::Metadata(key.clone(),values.filter(filter))
        }
    }

    pub fn compatible(&self, len: usize) -> bool {
        match self {
            Patina::Drawn(_,x) => x.compatible(len),
            Patina::Hotspot(hotspot,_) => { hotspot.compatible(len) },
            Patina::Metadata(_,values) => { values.compatible(len) }
        }
    }
}
