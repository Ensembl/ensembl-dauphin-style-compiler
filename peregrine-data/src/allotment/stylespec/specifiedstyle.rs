use std::collections::HashMap;

use crate::{CoordinateSystemVariety, allotment::style::style::{Indent, LeafAllotmentType}, LeafStyle, CoordinateSystem};

#[cfg_attr(any(test,debug_assertions),derive(Debug))]
#[derive(Clone)]
pub struct InheritableStyle {
    coord_system: Option<CoordinateSystemVariety>,
    bump_invisible: Option<bool>,
    reverse: Option<bool>,
    depth: Option<i8>,
    indent: Option<Indent>
}

impl InheritableStyle {
    pub(crate) fn empty() -> InheritableStyle {
        InheritableStyle {
            coord_system: None,
            bump_invisible: None,
            reverse: None,
            depth: None,
            indent: None
        }
    }

    pub(crate) fn new(spec: &HashMap<String,String>) -> InheritableStyle {
        let depth = spec.get("depth").map(|x| x.as_str());
        let depth = depth.map(|x| x.parse::<i8>().ok()).flatten();
        let (coord_system,reverse) = CoordinateSystem::build(spec);
        let indent = Indent::build(spec);
        let bump_invisible = spec.get("bump-width").map(|x| x.as_str() == "none");
        InheritableStyle {
            depth, coord_system, reverse, indent, bump_invisible
        }
    }

    pub(crate) fn override_style(&mut self, other: &InheritableStyle) {
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
        if other.bump_invisible.is_some() {
            self.bump_invisible = other.bump_invisible.clone();
        }
    }

    pub(crate) fn make(&self, style: &SpecifiedStyle) -> LeafStyle {
        let variety = self.coord_system.as_ref().unwrap_or(&CoordinateSystemVariety::Window).clone();
        let reverse = self.reverse.unwrap_or(false);
        LeafStyle {
            depth: self.depth.unwrap_or(0),
            coord_system: CoordinateSystem(variety,reverse),
            priority: style.priority,
            indent: self.indent.as_ref().unwrap_or(&Indent::None).clone(),
            bump_invisible: self.bump_invisible.unwrap_or(false)
        }
    }
}

#[cfg_attr(any(test,debug_assertions),derive(Debug))]
#[derive(Clone)]
pub struct SpecifiedStyle {
    pub allot_type: LeafAllotmentType,
    pub leaf: InheritableStyle,
    pub priority: i64
}

impl SpecifiedStyle {
    pub(crate) fn empty() -> SpecifiedStyle {
        SpecifiedStyle {
            allot_type: LeafAllotmentType::Leaf,
            leaf: InheritableStyle::empty(),
            priority: 0
        }
    }

    pub(crate) fn build(spec: &HashMap<String,String>) -> SpecifiedStyle {
        let allot_type = LeafAllotmentType::build(spec);
        let leaf = InheritableStyle::new(spec);
        let priority = spec.get("priority").map(|x| x.as_str());
        let priority = priority.map(|x| x.parse::<i64>().ok()).flatten().unwrap_or(0);
        SpecifiedStyle { allot_type, leaf, priority }
    }
}
