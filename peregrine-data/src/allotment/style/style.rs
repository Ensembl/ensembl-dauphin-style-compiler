use std::{collections::HashMap};

use crate::{CoordinateSystem, CoordinateSystemVariety, allotment::stylespec::specifiedstyle::InheritableStyle, shape::metadata::MetadataStyle};

#[cfg_attr(any(test,debug_assertions),derive(Debug))]
#[derive(Clone,PartialEq,Eq,Hash)]
pub enum Indent {
    None,
    Top,
    Left,
    Bottom,
    Right,
    Datum(String)
}

fn remove_bracketed(input: &str, prefix: &str, suffix: &str) -> Option<String> {
    if input.starts_with(prefix) && input.ends_with(suffix) {
        Some(input[prefix.len()..(input.len()-suffix.len())].to_string())
    } else {
        None
    }
}

impl Indent {
    pub(crate) fn build(spec: &HashMap<String,String>) -> Option<Indent> {
        let spec = spec.get("indent").map(|x| x.as_str());
        if let Some(spec) = spec {
            if let Some(datum) = remove_bracketed(spec,"datum(",")") {
                return Some(Indent::Datum(datum));
            }
        }
        match spec {
            Some("top") => Some(Indent::Top),
            Some("bottom") => Some(Indent::Bottom),
            Some("left") => Some(Indent::Left),
            Some("right") => Some(Indent::Right),
            Some("none") => Some(Indent::None),
            _ => None
        }
    }
}

#[cfg_attr(any(test,debug_assertions),derive(Debug))]
#[derive(Clone)]
pub enum LeafAllotmentType {
    Leaf
}

impl LeafAllotmentType {
    pub(crate) fn build(_spec: &HashMap<String,String>) -> LeafAllotmentType {
        LeafAllotmentType::Leaf
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
pub enum ContainerAllotmentType {
    Stack,
    Overlay,
    Bumper
}

impl ContainerAllotmentType {
    fn build(spec: &HashMap<String,String>) -> ContainerAllotmentType {
        let type_str = spec.get("type").map(|x| x.as_str());
        match type_str {
            Some("overlay") => ContainerAllotmentType::Overlay,
            Some("bumper") => ContainerAllotmentType::Bumper,
            _ => ContainerAllotmentType::Stack
        }    
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
pub struct Padding {
    pub padding_top: f64,
    pub padding_bottom: f64,
    pub min_height: f64,
    pub(crate) report: Option<MetadataStyle>
}

impl Padding {
    pub fn empty() -> Padding {
        Padding {
            padding_top: 0.,
            padding_bottom: 0.,
            min_height: 0.,
            report: None
        }
    }

    pub fn build(spec: &HashMap<String,String>) -> Padding {
        let padding_top = spec.get("padding-top").map(|x| x.as_str()).unwrap_or("0");
        let padding_top = padding_top.parse::<f64>().ok().unwrap_or(0.);
        let padding_bottom = spec.get("padding-bottom").map(|x| x.as_str()).unwrap_or("0");
        let padding_bottom = padding_bottom.parse::<f64>().ok().unwrap_or(0.);
        let min_height = spec.get("min-height").map(|x| x.as_str()).unwrap_or("0");
        let min_height = min_height.parse::<f64>().ok().unwrap_or(0.);
        let report = spec.get("report").map(|r| MetadataStyle::new(r));
        Padding {padding_top, padding_bottom, min_height, report }
    }
}

#[cfg_attr(any(test,debug_assertions),derive(Debug))]
#[derive(Clone)]
pub struct LeafStyle {
    pub coord_system: CoordinateSystem,
    pub depth: i8,
    pub priority: i64,
    pub indent: Indent,
    pub bump_invisible: bool
}

impl LeafStyle {
    pub fn dustbin() -> LeafStyle {
        LeafStyle {
            coord_system: CoordinateSystem(CoordinateSystemVariety::Dustbin,false),
            depth: 0,
            priority: 0,
            indent: Indent::None,
            bump_invisible: false
        }
    }

    pub fn default() -> LeafStyle {
        LeafStyle {
            coord_system: CoordinateSystem(CoordinateSystemVariety::Window,false),
            depth: 0,
            priority: 0,
            indent: Indent::None,
            bump_invisible: false
        }
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
pub struct ContainerAllotmentStyle {
    pub allot_type: ContainerAllotmentType,
    pub coord_system: CoordinateSystem,
    pub leaf: InheritableStyle,
    pub padding: Padding,
    pub priority: i64,
    pub ranged: bool,
    pub set_align: Option<String>,
    pub tracked_height: bool
}

impl ContainerAllotmentStyle {
    pub(crate) fn empty() -> ContainerAllotmentStyle {
        ContainerAllotmentStyle {
            allot_type: ContainerAllotmentType::Stack,
            coord_system: CoordinateSystem(CoordinateSystemVariety::Tracking,false),
            leaf: InheritableStyle::empty(),
            padding: Padding::empty(),
            priority: 0,
            ranged: false,
            set_align: None,
            tracked_height: false
        }
    }

    pub(crate) fn build(spec: &HashMap<String,String>) -> ContainerAllotmentStyle {
        let allot_type = ContainerAllotmentType::build(spec);
        let (coord_system,reverse) = CoordinateSystem::build(spec);
        let priority = spec.get("priority").map(|x| x.as_str());
        let priority = priority.map(|x| x.parse::<i64>().ok()).flatten().unwrap_or(0);
        let ranged = spec.get("extent").map(|x| x.as_str()).unwrap_or("wide") == "compact";
        let set_align = spec.get("set-datum").map(|x| x.to_string());
        let tracked_height = spec.get("height-adjust").map(|x| x.as_str()).unwrap_or("default") == "tracking";
        ContainerAllotmentStyle {
            allot_type,
            padding: Padding::build(spec),
            coord_system: CoordinateSystem::from_build(coord_system,reverse),
            leaf: InheritableStyle::new(spec),
            priority,
            ranged,
            set_align,
            tracked_height
        }
    }
}
