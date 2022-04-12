use std::{collections::HashMap, sync::{Arc, Mutex}};

use peregrine_toolkit::{lock, puzzle::{DelayedSetter, derived, cache_constant, commute_arc, constant, short_memoized, compose, StaticValue, promise_delayed, short_memoized_clonable, cache_constant_clonable}};

use crate::{allotment::{core::{allotmentmetadata::{AllotmentMetadataBuilder, AllotmentMetadataGroup}, aligner::Aligner, carriageoutput::BoxPositionContext}, style::{style::{ContainerAllotmentStyle}, allotmentname::{AllotmentName, AllotmentNamePart}}, boxes::boxtraits::Stackable, util::rangeused::RangeUsed}, CoordinateSystem};

use super::{boxtraits::{Coordinated, BuildSize, ContainerSpecifics}};

fn height(child_height: &StaticValue<f64>, tracked_height: &StaticValue<f64>, min_height: f64, padding_top: f64, padding_bottom: f64) -> StaticValue<f64> {
    cache_constant(compose(child_height.clone(),tracked_height.clone(),move |child_height,tracked_height| {
        let internal_height = child_height.max(min_height);
        let external_height = padding_top + internal_height + padding_bottom;
        external_height.max(tracked_height)
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
    height: StaticValue<Arc<f64>>,
    height_setter: DelayedSetter<'static,'static,Arc<f64>>,
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
            height: self.height.clone(),
            height_setter: self.height_setter.clone(),
            name: self.name.clone(),
            style: self.style.clone()
        }
    }
}

fn add_report(metadata: &mut AllotmentMetadataBuilder, in_values: &HashMap<String,String>, top: &StaticValue<f64>, height: &StaticValue<Arc<f64>>) {
    let mut values = HashMap::new();
    for (key,value) in in_values {
        values.insert(key.to_string(),constant(value.to_string()));
    }
    values.insert("offset".to_string(), derived(top.clone(),|v| v.to_string()));
    values.insert("height".to_string(), derived(height.clone(),|v| v.to_string()));
    let values = values.drain().map(|(k,v)| 
        (k,derived(cache_constant(short_memoized(v)),|x| (&**x).to_string()))
    ).collect::<HashMap<_,_>>();
    let group = AllotmentMetadataGroup::new(values);
    metadata.add(group);
}

impl Container {
    pub(crate) fn new<F>(name: &AllotmentNamePart, style: &ContainerAllotmentStyle, aligner: &Aligner, specifics: F) -> Container where F: ContainerSpecifics + 'static {
        let (top_setter,top) = promise_delayed();
        if let Some(datum) = &style.set_align {
            aligner.set_datum(datum,&top);
        }
        let (height_setter,height) = promise_delayed();
        Container {
            name: AllotmentName::from_part(name),
            specifics: Box::new(specifics),
            children: Arc::new(Mutex::new(vec![])),
            coord_system: style.coord_system.clone(),
            priority: style.priority,
            top_setter, top,
            height_setter, height,
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
        self.top_setter.set(value);
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
        let (tracked_height_setter,tracked_height) = promise_delayed();
        let (tracked_height_setter2,tracked_height2) = promise_delayed();
        
        let internal = self.specifics.build_reduce(prep,&input);
        let height = height(&internal,&tracked_height,self.style.padding.min_height,self.style.padding.padding_top,self.style.padding.padding_bottom);
        if self.style.tracked_height {
            prep.height_tracker.add(&self.name,&height,&mut prep.independent_answer);
            let global_piece = prep.height_tracker.get_piece(&self.name,&mut prep.independent_answer).clone();
            tracked_height_setter.set(global_piece.clone());

            let global_piece2 = prep.state_request.height_tracker_mut().add(&self.name,&height,&mut prep.independent_answer).clone();
            tracked_height_setter2.set(global_piece2);
        } else {
            tracked_height_setter.set(constant(0.));
            tracked_height_setter2.set(constant(0.));
        }
        if let Some(report) = &self.style.padding.report {
            let arc_height = derived(height.clone(),|x| Arc::new(x));
            add_report(&mut prep.metadata,report,&self.top,&arc_height);
        }
        self.height_setter.set(cache_constant(short_memoized_clonable(height.clone())));
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
