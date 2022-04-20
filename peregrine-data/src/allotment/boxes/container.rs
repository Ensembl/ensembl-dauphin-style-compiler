use std::{collections::HashMap, sync::{Arc, Mutex}};

use peregrine_toolkit::{lock, puzzle::{DelayedSetter, derived, cache_constant, commute_arc, constant, short_memoized, compose, StaticValue, promise_delayed, short_memoized_clonable, cache_constant_clonable, cache_constant_arc }};

use crate::{allotment::{core::{carriageoutput::BoxPositionContext, heighttracker, allotmentmetadata::LocalAllotmentMetadataBuilder}, style::{style::{ContainerAllotmentStyle}, allotmentname::{AllotmentName, AllotmentNamePart}}, boxes::boxtraits::Stackable, util::rangeused::RangeUsed}, CoordinateSystem};

use super::{boxtraits::{Coordinated, BuildSize, ContainerSpecifics}};

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
    //height: StaticValue<Arc<f64>>,
    //height_setter: DelayedSetter<'static,'static,Arc<f64>>,
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
            //height: self.height.clone(),
            //height_setter: self.height_setter.clone(),
            name: self.name.clone(),
            style: self.style.clone()
        }
    }
}

fn add_report(metadata: &mut LocalAllotmentMetadataBuilder, name: &AllotmentName, in_values: &HashMap<String,String>, top: &StaticValue<f64>, height: &StaticValue<Arc<f64>>) {
    metadata.set(name,"offset",derived(top.clone(),|v| v.to_string()));
    metadata.set(name,"height",derived(height.clone(),|v| v.to_string()));
    for (key,value) in in_values.iter() {
        let value = constant(value.to_string());
        metadata.set(name,key,value);
    }
}

impl Container {
    pub(crate) fn new<F>(name: &AllotmentNamePart, style: &ContainerAllotmentStyle, specifics: F) -> Container where F: ContainerSpecifics + 'static {
        let (top_setter,top) = promise_delayed();
        //let (height_setter,height) = promise_delayed();
        Container {
            name: AllotmentName::from_part(name),
            specifics: Box::new(specifics),
            children: Arc::new(Mutex::new(vec![])),
            coord_system: style.coord_system.clone(),
            priority: style.priority,
            top_setter, top,
            //height_setter, height,
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
        self.specifics.set_locate(prep,&draw_top,&mut kids);
        self.top_setter.set(value.clone());
        if let Some(datum) = &self.style.set_align {
            prep.state_request.aligner_mut().set(datum,value);
        }
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
            height_tracker.set(&self.name,internal_height,&mut prep.independent_answer);
            height_tracker.global(&self.name).clone()
        } else {
            internal_height
        };
        if let Some(report) = &self.style.padding.report {
            let arc_height = derived(height.clone(),|x| Arc::new(x));
            add_report(prep.state_request.metadata_mut(),&self.name,report,&self.top,&arc_height);
        }
        //self.height_setter.set(cache_constant(short_memoized_clonable(height.clone())));
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
