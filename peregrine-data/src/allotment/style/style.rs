use std::{collections::HashMap, sync::Arc};

use crate::{CoordinateSystem, CoordinateSystemVariety};

#[cfg_attr(any(test,debug_assertions),derive(Debug))]
#[derive(Clone)]
pub enum LeafAllotmentType {
    Leaf
}

#[cfg_attr(any(test,debug_assertions),derive(Debug))]
#[derive(Clone)]
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
    fn build(spec: &HashMap<String,String>) -> Option<Indent> {
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

impl LeafAllotmentType {
    fn build(_spec: &HashMap<String,String>) -> LeafAllotmentType {
        LeafAllotmentType::Leaf
    }
}

fn parse_report_value(input: &str) -> Option<Arc<HashMap<String,String>>> {
    let parts = input.split(";").collect::<Vec<_>>();
    let mut out = HashMap::new();
    for item in &parts {
        let (key,value) = if let Some(eq_at) = item.find("=") {
            let (k,v) = item.split_at(eq_at);
            (k,&v[1..])
        } else {
            ("type",*item)
        };
        out.insert(key.to_string(),value.to_string());
    }
    Some(Arc::new(out))
}

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
pub struct Padding {
    pub padding_top: f64,
    pub padding_bottom: f64,
    pub min_height: f64,
    pub report: Option<Arc<HashMap<String,String>>>
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
        let report = spec.get("report").and_then(|r| parse_report_value(r));
        Padding {padding_top, padding_bottom, min_height, report }
    }
}

#[cfg_attr(any(test,debug_assertions),derive(Debug))]
#[derive(Clone)]
pub struct LeafInheritStyle {
    coord_system: Option<CoordinateSystemVariety>,
    reverse: Option<bool>,
    depth: Option<i8>,
    indent: Option<Indent>
}

impl LeafInheritStyle {
    pub(crate) fn empty() -> LeafInheritStyle {
        LeafInheritStyle {
            coord_system: None,
            reverse: None,
            depth: None,
            indent: None
        }
    }

    fn new(spec: &HashMap<String,String>) -> LeafInheritStyle {
        let depth = spec.get("depth").map(|x| x.as_str());
        let depth = depth.map(|x| x.parse::<i8>().ok()).flatten();
        let (coord_system,reverse) = CoordinateSystem::build(spec);
        let indent = Indent::build(spec);
        LeafInheritStyle {
            depth, coord_system, reverse, indent
        }
    }

    pub(crate) fn override_style(&mut self, other: &LeafInheritStyle) {
        if other.depth.is_some() {
            self.depth = other.depth.clone();
        }
        if other.coord_system.is_some() {
            self.coord_system = other.coord_system.clone();
        }
        if other.reverse.is_some() {
            self.reverse = other.reverse.clone();
        }
        if other.indent.is_some() {
            self.indent = other.indent.clone();
        }
    }

    pub(crate) fn make(&self, style: &LeafAllotmentStyle) -> LeafCommonStyle {
        let variety = self.coord_system.as_ref().unwrap_or(&CoordinateSystemVariety::Window).clone();
        let reverse = self.reverse.unwrap_or(false);
        LeafCommonStyle {
            depth: self.depth.unwrap_or(0),
            coord_system: CoordinateSystem(variety,reverse),
            priority: style.priority,
            indent: self.indent.as_ref().unwrap_or(&Indent::None).clone()
        }
    }
}

#[cfg_attr(any(test,debug_assertions),derive(Debug))]
#[derive(Clone)]
pub struct LeafCommonStyle {
    pub coord_system: CoordinateSystem,
    pub depth: i8,
    pub priority: i64,
    pub indent: Indent
}

impl LeafCommonStyle {
    pub fn dustbin() -> LeafCommonStyle {
        LeafCommonStyle {
            coord_system: CoordinateSystem(CoordinateSystemVariety::Dustbin,false),
            depth: 0,
            priority: 0,
            indent: Indent::None
        }
    }

    pub fn default() -> LeafCommonStyle {
        LeafCommonStyle {
            coord_system: CoordinateSystem(CoordinateSystemVariety::Window,false),
            depth: 0,
            priority: 0,
            indent: Indent::None
        }
    }
}

#[cfg_attr(any(test,debug_assertions),derive(Debug))]
#[derive(Clone)]
pub struct LeafAllotmentStyle {
    pub allot_type: LeafAllotmentType,
    pub leaf: LeafInheritStyle,
    pub priority: i64
}

impl LeafAllotmentStyle {
    pub(crate) fn empty() -> LeafAllotmentStyle {
        LeafAllotmentStyle {
            allot_type: LeafAllotmentType::Leaf,
            leaf: LeafInheritStyle::empty(),
            priority: 0
        }
    }

    pub(crate) fn build(spec: &HashMap<String,String>) -> LeafAllotmentStyle {
        let allot_type = LeafAllotmentType::build(spec);
        let leaf = LeafInheritStyle::new(spec);
        let priority = spec.get("priority").map(|x| x.as_str());
        let priority = priority.map(|x| x.parse::<i64>().ok()).flatten().unwrap_or(0);
        LeafAllotmentStyle { allot_type, leaf, priority }
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
pub struct ContainerAllotmentStyle {
    pub allot_type: ContainerAllotmentType,
    pub coord_system: CoordinateSystem,
    pub leaf: LeafInheritStyle,
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
            leaf: LeafInheritStyle::empty(),
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
            leaf: LeafInheritStyle::new(spec),
            priority,
            ranged,
            set_align,
            tracked_height
        }
    }
}
