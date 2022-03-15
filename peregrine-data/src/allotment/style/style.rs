use std::collections::HashMap;

use crate::{CoordinateSystem, CoordinateSystemVariety};

#[cfg_attr(any(test,debug_assertions),derive(Debug))]
#[derive(Clone)]
pub enum LeafAllotmentType {
    Leaf
}

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
    priority: Option<i64>,
    coord_system: Option<CoordinateSystem>,
    depth: Option<i8>
}

impl LeafInheritStyle {
    pub(crate) fn empty() -> LeafInheritStyle {
        LeafInheritStyle {
            priority: None,
            coord_system: None,
            depth: None
        }
    }

    fn new(spec: &HashMap<String,String>) -> LeafInheritStyle {
        let priority = spec.get("priority").map(|x| x.as_str());
        let priority = priority.map(|x| x.parse::<i64>().ok()).flatten();
        let depth = spec.get("depth").map(|x| x.as_str());
        let depth = depth.map(|x| x.parse::<i8>().ok()).flatten();
        let coord_system = CoordinateSystem::build(spec);
        LeafInheritStyle {
            priority, depth,
            coord_system
        }
    }

    pub(crate) fn override_style(&mut self, other: &LeafInheritStyle) {
        if other.priority.is_some() {
            self.priority = other.priority.clone();
        }
        if other.depth.is_some() {
            self.depth = other.depth.clone();
        }
        if other.coord_system.is_some() {
            self.coord_system = other.coord_system.clone();
        }
    }

    pub(crate) fn make(&self) -> LeafCommonStyle {
        LeafCommonStyle {
            depth: self.depth.unwrap_or(0),
            priority: self.priority.unwrap_or(0),
            coord_system: self.coord_system.as_ref().unwrap_or(&CoordinateSystem(CoordinateSystemVariety::Window,false)).clone()
        }
    }
}

#[cfg_attr(any(test,debug_assertions),derive(Debug))]
#[derive(Clone)]
pub struct LeafCommonStyle {
    pub priority: i64,
    pub coord_system: CoordinateSystem,
    pub depth: i8
}

impl LeafCommonStyle {
    pub fn dustbin() -> LeafCommonStyle {
        LeafCommonStyle {
            priority: 0,
            coord_system: CoordinateSystem(CoordinateSystemVariety::Dustbin,false),
            depth: 0
        }
    }

    pub fn default() -> LeafCommonStyle {
        LeafCommonStyle {
            priority: 0,
            coord_system: CoordinateSystem(CoordinateSystemVariety::Window,false),
            depth: 0
        }
    }
}

#[cfg_attr(any(test,debug_assertions),derive(Debug))]
#[derive(Clone)]
pub struct LeafAllotmentStyle {
    pub allot_type: LeafAllotmentType,
    pub leaf: LeafInheritStyle
}

impl LeafAllotmentStyle {
    pub(crate) fn empty() -> LeafAllotmentStyle {
        LeafAllotmentStyle {
            allot_type: LeafAllotmentType::Leaf,
            leaf: LeafInheritStyle::empty()
        }
    }

    pub(crate) fn build(spec: &HashMap<String,String>) -> LeafAllotmentStyle {
        let allot_type = LeafAllotmentType::build(spec);
        let leaf = LeafInheritStyle::new(spec);
        LeafAllotmentStyle { allot_type, leaf }
    }
}

#[derive(Clone)]
pub struct ContainerAllotmentStyle {
    pub allot_type: ContainerAllotmentType,
    pub coord_system: CoordinateSystem,
    pub leaf: LeafInheritStyle,
    pub padding: Padding
}

impl ContainerAllotmentStyle {
    pub(crate) fn empty() -> ContainerAllotmentStyle {
        ContainerAllotmentStyle {
            allot_type: ContainerAllotmentType::Stack,
            coord_system: CoordinateSystem(CoordinateSystemVariety::Tracking,false),
            leaf: LeafInheritStyle::empty(),
            padding: Padding::empty()
        }
    }

    pub(crate) fn build(spec: &HashMap<String,String>) -> ContainerAllotmentStyle {
        let allot_type = ContainerAllotmentType::build(spec);
        ContainerAllotmentStyle {
            allot_type,
            padding: Padding::build(spec),
            coord_system: CoordinateSystem::build(spec).unwrap_or(CoordinateSystem(CoordinateSystemVariety::Tracking,false)),
            leaf: LeafInheritStyle::new(spec)
        }
    }
}
