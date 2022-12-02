use std::collections::HashMap;
use crate::{CoordinateSystem, allotment::boxes::leaf::AuxLeaf};

fn remove_bracketed(input: &str, prefix: &str, suffix: &str) -> Option<String> {
    if input.starts_with(prefix) && input.ends_with(suffix) {
        Some(input[prefix.len()..(input.len()-suffix.len())].to_string())
    } else {
        None
    }
}

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
pub struct InheritableLeafStyle {
    coord_system: Option<CoordinateSystem>,
    bump_invisible: Option<bool>,
    depth: Option<i8>,
    indent: Option<Indent>
}

impl InheritableLeafStyle {
    pub(crate) fn empty() -> InheritableLeafStyle {
        InheritableLeafStyle {
            coord_system: None,
            bump_invisible: None,
            depth: None,
            indent: None
        }
    }

    pub(crate) fn new(spec: &HashMap<String,String>) -> InheritableLeafStyle {
        let depth = spec.get("depth").map(|x| x.as_str());
        let depth = depth.map(|x| x.parse::<i8>().ok()).flatten();
        let coord_system = CoordinateSystem::build(spec);
        let indent = Indent::build(spec);
        let bump_invisible = spec.get("bump-width").map(|x| x.as_str() == "none");
        InheritableLeafStyle {
            depth, coord_system, indent, bump_invisible
        }
    }

    pub(crate) fn override_style(&mut self, other: &InheritableLeafStyle) {
        if other.depth.is_some() {
            self.depth = other.depth.clone();
        }
        if other.coord_system.is_some() {
            self.coord_system = other.coord_system.clone();
        }
        if other.indent.is_some() {
            self.indent = other.indent.clone();
        }
        if other.bump_invisible.is_some() {
            self.bump_invisible = other.bump_invisible.clone();
        }
    }

    pub(crate) fn make(&self, uninh: &UninheritableLeafStyle) -> LeafStyle {
        let coord_system = self.coord_system.as_ref().unwrap_or(&CoordinateSystem::Window).clone();
        LeafStyle {
            aux: AuxLeaf {
                depth: self.depth.unwrap_or(0),
                coord_system,
            },
            priority: uninh.priority,
            indent: self.indent.as_ref().unwrap_or(&Indent::None).clone(),
            bump_invisible: self.bump_invisible.unwrap_or(false)
        }
    }
}

#[cfg_attr(any(test,debug_assertions),derive(Debug))]
#[derive(Clone)]
pub struct UninheritableLeafStyle {
    pub(crate) priority: i64
}

impl UninheritableLeafStyle {
    pub(crate) fn build(spec: &HashMap<String,String>) -> UninheritableLeafStyle {
        let priority = spec.get("priority").map(|x| x.as_str());
        let priority = priority.map(|x| x.parse::<i64>().ok()).flatten().unwrap_or(0);
        UninheritableLeafStyle { priority }
    }
}

#[cfg_attr(any(test,debug_assertions),derive(Debug))]
#[derive(Clone)]
pub struct LeafStyle {
    pub(crate) aux: AuxLeaf,
    pub(crate) priority: i64,
    pub(crate) indent: Indent,
    pub(crate) bump_invisible: bool
}

impl LeafStyle {
    pub(crate) fn dustbin() -> LeafStyle {
        LeafStyle {
            aux: AuxLeaf {
                coord_system: CoordinateSystem::Dustbin,
                depth: 0,
            },
            priority: 0,
            indent: Indent::None,
            bump_invisible: false
        }
    }
}
