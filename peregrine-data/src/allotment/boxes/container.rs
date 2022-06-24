use std::{collections::HashMap, sync::{Arc, Mutex}};
use peregrine_toolkit::{lock, puzzle::{DelayedSetter, derived, cache_constant, commute_arc, constant, StaticValue, promise_delayed, short_memoized_clonable, cache_constant_clonable }, eachorevery::eoestruct::StructTemplate};
use crate::{allotment::{core::{allotmentname::{AllotmentName, AllotmentNamePart}, boxtraits::{ContainerSpecifics, Coordinated, BuildSize, Stackable}, boxpositioncontext::BoxPositionContext}, style::{style::{ContainerAllotmentStyle}}, util::rangeused::RangeUsed, globals::allotmentmetadata::LocalAllotmentMetadataBuilder}, CoordinateSystem, shape::metadata::MetadataStyle};

fn internal_height(child_height: &StaticValue<f64>, min_height: f64, padding_top: f64, padding_bottom: f64) -> StaticValue<f64> {
    cache_constant(derived(child_height.clone(),move |child_height| {
        let internal_height = child_height.max(min_height);
        padding_top + internal_height + padding_bottom
    })).dearc()
}

pub struct Container {
    specifics: Box<dyn ContainerSpecifics>,
    children: Arc<Mutex<Vec<Box<dyn Stackable>>>>,
    coord_system: CoordinateSystem,
    priority: i64,
    /* incoming variables */
    top: StaticValue<f64>,
    top_setter: DelayedSetter<'static,'static,f64>,
    /* outgoing variables */
    name: AllotmentName,
    style: Arc<ContainerAllotmentStyle>,
}

impl Clone for Container {
    fn clone(&self) -> Self {
        Self {
            specifics: self.specifics.cloned(),
            children: self.children.clone(),
            coord_system: self.coord_system.clone(),
            priority: self.priority.clone(),
            top: self.top.clone(),
            top_setter: self.top_setter.clone(),
            name: self.name.clone(),
            style: self.style.clone()
        }
    }
}

fn add_report(metadata: &mut LocalAllotmentMetadataBuilder, name: &AllotmentName, in_values: &MetadataStyle, top: &StaticValue<f64>, height: &StaticValue<Arc<f64>>) {
    metadata.set(name,"offset",derived(top.clone(),|v| StructTemplate::new_number(v)),false);
    metadata.set(name,"height",derived(height.clone(),|v| StructTemplate::new_number(*v)),false);
    for (key,value) in in_values.iter() {
        let value = constant(StructTemplate::new_string(value.to_string()));
        metadata.set(name,key,value,false);
    }
    if in_values.reporting() {
        metadata.set_reporting(name);
    }
}

impl Container {
    pub(crate) fn new<F>(name: &AllotmentNamePart, style: &ContainerAllotmentStyle, specifics: F) -> Container where F: ContainerSpecifics + 'static {
        let (top_setter,top) = promise_delayed();
        Container {
            name: AllotmentName::from_part(name),
            specifics: Box::new(specifics),
            children: Arc::new(Mutex::new(vec![])),
            coord_system: style.coord_system.clone(),
            priority: style.priority,
            top_setter, top,
            style: Arc::new(style.clone())
        }
    }

    pub(crate) fn add_child(&mut self, child: &dyn Stackable) {
        lock!(self.children).push(child.cloned());
    }
}

impl Coordinated for Container {
    fn coordinate_system(&self) -> &CoordinateSystem { &self.coord_system }
}

impl Stackable for Container {
    fn locate(&mut self, prep: &mut BoxPositionContext, value: &StaticValue<f64>) {
        let value = cache_constant_clonable(short_memoized_clonable(value.clone()));
        let mut children = lock!(self.children);
        let mut kids = children.iter_mut().collect::<Vec<_>>();
        let padding_top = self.style.padding.padding_top;
        let draw_top = cache_constant(derived(value.clone(),move |top| top+padding_top)).dearc();
        self.top_setter.set(value.clone());
        if let Some(datum) = &self.style.set_align {
            prep.state_request.aligner_mut().set(datum,value);
        }
        self.specifics.set_locate(prep,&draw_top,&mut kids);
    }

    fn name(&self) -> &AllotmentName { &self.name }

    fn build(&mut self, prep: &mut BoxPositionContext) -> BuildSize {
        let mut ranges = vec![];
        let mut children = lock!(self.children);
        let mut input = vec![];
        for child in &mut *children {
            let size = child.build(prep);
            let range = derived(size.range.clone(),|x| Arc::new(x));
            ranges.push(range);
            input.push((&*child,size));
        }
        let kids = self.specifics.build_reduce(prep,&input);
        let internal_height = internal_height(&kids,self.style.padding.min_height,self.style.padding.padding_top,self.style.padding.padding_bottom);
        let height = if self.style.tracked_height {
            let height_tracker = prep.state_request.height_tracker_mut();
            height_tracker.set(&self.name,internal_height);
            height_tracker.global(&self.name).clone()
        } else {
            internal_height
        };
        if let Some(report) = &self.style.padding.report {
            let arc_height = derived(height.clone(),|x| Arc::new(x));
            add_report(prep.state_request.metadata_mut(),&self.name,report,&self.top,&arc_height);
        }
        let range = commute_arc(&ranges,Arc::new(RangeUsed::None), Arc::new(|x,y| (*x).merge(&*y))).dearc();
        BuildSize {
            name: self.name.clone(),
            height,
            range
        }
    }

    fn priority(&self) -> i64 { self.priority }

    fn cloned(&self) -> Box<dyn Stackable> { Box::new(self.clone()) }
}
