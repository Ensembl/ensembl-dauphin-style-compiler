use std::collections::HashMap;

use crate::{CoordinateSystem, CoordinateSystemVariety};

pub enum LeafAllotmentType {
    Leaf
}

#[derive(Debug)]
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

pub struct Padding {
    pub padding_top: f64,
    pub padding_bottom: f64,
    pub min_height: f64,
    pub indent: f64
}

impl Padding {
    pub fn empty() -> Padding {
        Padding {
            padding_top: 0.,
            padding_bottom: 0.,
            min_height: 0.,
            indent: 0.
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
        Padding {padding_top, padding_bottom, min_height, indent }
    }
}

#[cfg_attr(test,derive(Debug))]
#[derive(Clone)]
pub struct LeafCommonStyle {
    pub dustbin: bool,
    pub priority: i64,
    pub coord_system: CoordinateSystem,
    pub depth: i8
}

impl LeafCommonStyle {
    pub fn dustbin() -> LeafCommonStyle {
        LeafCommonStyle {
            dustbin: true,
            priority: 0,
            coord_system: CoordinateSystem(CoordinateSystemVariety::Dustbin,false),
            depth: 0
        }
    }

    pub fn default() -> LeafCommonStyle {
        LeafCommonStyle {
            dustbin: false,
            priority: 0,
            coord_system: CoordinateSystem(CoordinateSystemVariety::Tracking,false),
            depth: 0
        }
    }

    pub fn build(spec: &HashMap<String,String>) -> LeafCommonStyle {
        let coord_name = spec.get("system").map(|x| x.as_str()).unwrap_or("");
        let coord_direction = spec.get("direction").map(|x| x.as_str()).unwrap_or("");
        let coord_system = CoordinateSystem::from_string(coord_name,coord_direction);
        let priority = spec.get("priority").map(|x| x.as_str()).unwrap_or("0");
        let priority = priority.parse::<i64>().ok().unwrap_or(0);
        let depth = spec.get("depth").map(|x| x.as_str()).unwrap_or("0");
        let depth = depth.parse::<i8>().ok().unwrap_or(0);
        LeafCommonStyle {
            dustbin: coord_system.is_dustbin(),
            priority, depth, coord_system
        }
    }
}
pub struct LeafAllotmentStyle {
    pub allot_type: LeafAllotmentType,
    pub leaf: LeafCommonStyle,
}

impl LeafAllotmentStyle {
    pub(super) fn empty() -> LeafAllotmentStyle {
        LeafAllotmentStyle {
            allot_type: LeafAllotmentType::Leaf,
            leaf: LeafCommonStyle::default()
        }
    }

    pub(super) fn build(spec: &HashMap<String,String>) -> LeafAllotmentStyle {
        let allot_type = LeafAllotmentType::build(spec);
        let leaf = LeafCommonStyle::build(spec);
        LeafAllotmentStyle { allot_type, leaf }
    }
}

pub struct ContainerAllotmentStyle {
    pub allot_type: ContainerAllotmentType,
    pub padding: Padding
}

impl ContainerAllotmentStyle {
    pub(super) fn empty() -> ContainerAllotmentStyle {
        ContainerAllotmentStyle {
            allot_type: ContainerAllotmentType::Stack,
            padding: Padding::empty()
        }
    }

    pub(super) fn build(spec: &HashMap<String,String>) -> ContainerAllotmentStyle {
        let allot_type = ContainerAllotmentType::build(spec);
        println!("type {:?}",allot_type);
        ContainerAllotmentStyle { allot_type, padding: Padding::build(spec) }
    }
}
