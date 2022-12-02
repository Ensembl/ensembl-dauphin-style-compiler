use std::{collections::HashMap};
use crate::{CoordinateSystem, shape::metadata::MetadataStyle};
use super::leafstyle::InheritableLeafStyle;

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
    pub(crate) padding_top: f64,
    pub(crate) padding_bottom: f64,
    pub(crate) min_height: f64,
    pub(crate) report: Option<MetadataStyle>
}

impl Padding {
    pub(crate) fn build(spec: &HashMap<String,String>) -> Padding {
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

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
pub struct ContainerStyle {
    pub(crate) allot_type: ContainerAllotmentType,
    pub(crate) coord_system: CoordinateSystem,
    pub(crate) leaf: InheritableLeafStyle,
    pub(crate) padding: Padding,
    pub(crate) priority: i64,
    pub(crate) set_align: Option<String>,
    pub(crate) tracked_height: bool
}

impl ContainerStyle {
    pub(crate) fn build(spec: &HashMap<String,String>) -> ContainerStyle {
        let allot_type = ContainerAllotmentType::build(spec);
        let coord_system = CoordinateSystem::build(spec).unwrap_or(CoordinateSystem::Window);
        let priority = spec.get("priority").map(|x| x.as_str());
        let priority = priority.map(|x| x.parse::<i64>().ok()).flatten().unwrap_or(0);
        let set_align = spec.get("set-datum").map(|x| x.to_string());
        let tracked_height = spec.get("height-adjust").map(|x| x.as_str()).unwrap_or("default") == "tracking";
        ContainerStyle {
            allot_type,
            padding: Padding::build(spec),
            coord_system,
            leaf: InheritableLeafStyle::new(spec),
            priority,
            set_align,
            tracked_height
        }
    }
}
