use std::collections::HashMap;

use crate::{CoordinateSystem, CoordinateSystemVariety};

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

#[cfg_attr(test,derive(Debug))]
#[derive(Clone)]
/* style only specifiable at top level, but only used in leaves */
pub struct TopStyle {
    pub coord_system: CoordinateSystem
}

impl TopStyle {
    pub fn dustbin() -> TopStyle {
        TopStyle {
            coord_system: CoordinateSystem(CoordinateSystemVariety::Dustbin,false),
        }
    }

    pub fn default() -> TopStyle {
        TopStyle {
            coord_system: CoordinateSystem(CoordinateSystemVariety::Tracking,false),
        }
    }

    pub(crate) fn build(spec: &HashMap<String,String>) -> TopStyle {
        let coord_name = spec.get("system").map(|x| x.as_str()).unwrap_or("");
        let coord_direction = spec.get("direction").map(|x| x.as_str()).unwrap_or("");
        let coord_system = CoordinateSystem::from_string(coord_name,coord_direction);
        TopStyle {
            coord_system
        }
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

#[cfg_attr(test,derive(Debug))]
#[derive(Clone)]
pub struct LeafCommonStyle {
    pub dustbin: bool,
    pub priority: i64,
    pub top_style: TopStyle,
    pub depth: i8
}

impl LeafCommonStyle {
    pub fn dustbin() -> LeafCommonStyle {
        LeafCommonStyle {
            dustbin: true,
            priority: 0,
            top_style: TopStyle::dustbin(),
            depth: 0
        }
    }

    pub fn default() -> LeafCommonStyle {
        LeafCommonStyle {
            dustbin: false,
            priority: 0,
            top_style: TopStyle::default(),
            depth: 0
        }
    }

    pub fn build(spec: &HashMap<String,String>, top_style: Option<&TopStyle>) -> LeafCommonStyle {
        let priority = spec.get("priority").map(|x| x.as_str()).unwrap_or("0");
        let priority = priority.parse::<i64>().ok().unwrap_or(0);
        let depth = spec.get("depth").map(|x| x.as_str()).unwrap_or("0");
        let depth = depth.parse::<i8>().ok().unwrap_or(0);
        let top_style = top_style.cloned().unwrap_or_else(|| TopStyle::default());
        LeafCommonStyle {
            dustbin: top_style.coord_system.is_dustbin(),
            priority, depth,
            top_style
        }
    }
}

#[derive(Clone)]
pub struct LeafAllotmentStyle {
    pub allot_type: LeafAllotmentType,
    pub leaf: LeafCommonStyle,
}

impl LeafAllotmentStyle {
    pub(crate) fn empty() -> LeafAllotmentStyle {
        LeafAllotmentStyle {
            allot_type: LeafAllotmentType::Leaf,
            leaf: LeafCommonStyle::default()
        }
    }

    pub(crate) fn build(spec: &HashMap<String,String>, topstyle: Option<&TopStyle>) -> LeafAllotmentStyle {
        let allot_type = LeafAllotmentType::build(spec);
        let leaf = LeafCommonStyle::build(spec,topstyle);
        LeafAllotmentStyle { allot_type, leaf }
    }
}

#[derive(Clone)]
pub struct ContainerAllotmentStyle {
    pub allot_type: ContainerAllotmentType,
    pub padding: Padding
}

impl ContainerAllotmentStyle {
    pub(crate) fn empty() -> ContainerAllotmentStyle {
        ContainerAllotmentStyle {
            allot_type: ContainerAllotmentType::Stack,
            padding: Padding::empty()
        }
    }

    pub(crate) fn build(spec: &HashMap<String,String>) -> ContainerAllotmentStyle {
        let allot_type = ContainerAllotmentType::build(spec);
        ContainerAllotmentStyle { allot_type, padding: Padding::build(spec) }
    }
}
