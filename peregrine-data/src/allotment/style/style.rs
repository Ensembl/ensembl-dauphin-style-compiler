use std::collections::HashMap;

use crate::{CoordinateSystem, CoordinateSystemVariety};

#[cfg_attr(any(test,debug_assertions),derive(Debug))]
#[derive(Clone)]
pub enum LeafAllotmentType {
    Leaf
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
    fn build(spec: &HashMap<String,String>) -> LeafAllotmentType {
        LeafAllotmentType::Leaf
    }
}

#[derive(Clone)]
pub struct Padding {
    pub padding_top: f64,
    pub padding_bottom: f64,
    pub min_height: f64,
    pub indent: f64,
    pub report: Option<String>
}

impl Padding {
    pub fn empty() -> Padding {
        Padding {
            padding_top: 0.,
            padding_bottom: 0.,
            min_height: 0.,
            indent: 0.,
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
        let indent = spec.get("indent").map(|x| x.as_str()).unwrap_or("0");
        let indent = indent.parse::<f64>().ok().unwrap_or(0.);
        let report = spec.get("report").cloned();
        Padding {padding_top, padding_bottom, min_height, indent, report }
    }
}

#[cfg_attr(any(test,debug_assertions),derive(Debug))]
#[derive(Clone)]
pub struct LeafInheritStyle {
    coord_system: Option<CoordinateSystemVariety>,
    reverse: Option<bool>,
    depth: Option<i8>
}

impl LeafInheritStyle {
    pub(crate) fn empty() -> LeafInheritStyle {
        LeafInheritStyle {
            coord_system: None,
            reverse: None,
            depth: None
        }
    }

    fn new(spec: &HashMap<String,String>) -> LeafInheritStyle {
        let depth = spec.get("depth").map(|x| x.as_str());
        let depth = depth.map(|x| x.parse::<i8>().ok()).flatten();
        let (coord_system,reverse) = CoordinateSystem::build(spec);
        LeafInheritStyle {
            depth, coord_system, reverse
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
    }

    pub(crate) fn make(&self, style: &LeafAllotmentStyle) -> LeafCommonStyle {
        let variety = self.coord_system.as_ref().unwrap_or(&CoordinateSystemVariety::Window).clone();
        let reverse = self.reverse.unwrap_or(false);
        LeafCommonStyle {
            depth: self.depth.unwrap_or(0),
            coord_system: CoordinateSystem(variety,reverse),
            priority: style.priority
        }
    }
}

#[cfg_attr(any(test,debug_assertions),derive(Debug))]
#[derive(Clone)]
pub struct LeafCommonStyle {
    pub coord_system: CoordinateSystem,
    pub depth: i8,
    pub priority: i64
}

impl LeafCommonStyle {
    pub fn dustbin() -> LeafCommonStyle {
        LeafCommonStyle {
            coord_system: CoordinateSystem(CoordinateSystemVariety::Dustbin,false),
            depth: 0,
            priority: 0
        }
    }

    pub fn default() -> LeafCommonStyle {
        LeafCommonStyle {
            coord_system: CoordinateSystem(CoordinateSystemVariety::Window,false),
            depth: 0,
            priority: 0
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

#[derive(Clone)]
pub struct ContainerAllotmentStyle {
    pub allot_type: ContainerAllotmentType,
    pub coord_system: CoordinateSystem,
    pub leaf: LeafInheritStyle,
    pub padding: Padding,
    pub priority: i64
}

impl ContainerAllotmentStyle {
    pub(crate) fn empty() -> ContainerAllotmentStyle {
        ContainerAllotmentStyle {
            allot_type: ContainerAllotmentType::Stack,
            coord_system: CoordinateSystem(CoordinateSystemVariety::Tracking,false),
            leaf: LeafInheritStyle::empty(),
            padding: Padding::empty(),
            priority: 0
        }
    }

    pub(crate) fn build(spec: &HashMap<String,String>) -> ContainerAllotmentStyle {
        let allot_type = ContainerAllotmentType::build(spec);
        let (coord_system,reverse) = CoordinateSystem::build(spec);
        let priority = spec.get("priority").map(|x| x.as_str());
        let priority = priority.map(|x| x.parse::<i64>().ok()).flatten().unwrap_or(0);
        ContainerAllotmentStyle {
            allot_type,
            padding: Padding::build(spec),
            coord_system: CoordinateSystem::from_build(coord_system,reverse),
            leaf: LeafInheritStyle::new(spec),
            priority
        }
    }
}
