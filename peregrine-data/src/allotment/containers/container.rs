use std::{sync::{Arc}};
use peregrine_toolkit::{puzzle::{DelayedSetter, derived, cache_constant, constant, StaticValue, promise_delayed, short_memoized_clonable, cache_constant_clonable, StaticAnswer }, eachorevery::eoestruct::StructTemplate};
use crate::{allotment::{core::{allotmentname::{AllotmentName}, boxpositioncontext::BoxPositionContext}, style::{containerstyle::{ContainerStyle}}, util::rangeused::RangeUsed, globals::allotmentmetadata::LocalAllotmentMetadataBuilder, style::{styletree::StyleTree}, leafs::{floating::FloatingLeaf, anchored::AnchoredLeaf}, layout::stylebuilder::{ContainerOrLeaf, BuildSize}}, shape::metadata::MetadataStyle, CoordinateSystem, LeafRequest};
use super::{haskids::HasKids};

pub(crate) trait ContainerSpecifics {
    fn build_reduce(&self, prep: &mut BoxPositionContext, children: &[(&Box<dyn ContainerOrLeaf>,BuildSize)]) -> StaticValue<f64>;
    fn set_locate(&self, prep: &mut BoxPositionContext, top: &StaticValue<f64>, children: &mut [&mut Box<dyn ContainerOrLeaf>]);
}

fn internal_height(child_height: &StaticValue<f64>, min_height: f64, padding_top: f64, padding_bottom: f64) -> StaticValue<f64> {
    cache_constant(derived(child_height.clone(),move |child_height| {
        let internal_height = child_height.max(min_height);
        padding_top + internal_height + padding_bottom
    })).derc()
}

#[derive(PartialEq,Eq,Hash,Clone)]
pub(super) enum ChildKeys {
    Container(String),
    Leaf(String)
}

pub struct Container {
    specifics: Box<dyn ContainerSpecifics>,
    coord_system: CoordinateSystem,
    kids: HasKids,
    priority: i64,
    /* incoming variables */
    top: StaticValue<f64>,
    top_setter: DelayedSetter<'static,'static,f64>,
    /* outgoing variables */
    name: AllotmentName,
    style: Arc<ContainerStyle>,
}

fn add_report(metadata: &mut LocalAllotmentMetadataBuilder, name: &AllotmentName, in_values: &MetadataStyle, top: &StaticValue<f64>, height: &StaticValue<Arc<f64>>) {
    metadata.set(name,"offset",derived(top.clone(),|v| StructTemplate::new_number(v)),None);
    metadata.set(name,"height",derived(height.clone(),|v| StructTemplate::new_number(*v)),None);
    for (key,value) in in_values.iter() {
        let value = constant(StructTemplate::new_string(value.to_string()));
        metadata.set(name,key,value,None);
    }
    metadata.set_reporting(name,in_values.reporting());
}

impl Container {
    pub(crate) fn new(name: &AllotmentName, style: &ContainerStyle, specifics: Box<dyn ContainerSpecifics + 'static>) -> Container {
        let (top_setter,top) = promise_delayed();
        Container {
            name: name.clone(),
            specifics,
            kids: HasKids::new(),
            coord_system: style.coord_system.clone(),
            priority: style.priority,
            top_setter, top,
            style: Arc::new(style.clone())
        }
    }
}

impl ContainerOrLeaf for Container {
    fn anchor_leaf(&self, _answer_index: &StaticAnswer) -> Option<AnchoredLeaf> { None }
    
    fn get_leaf(&mut self, pending: &LeafRequest, cursor: usize, styles: &Arc<StyleTree>) -> FloatingLeaf {
        self.kids.get_leaf(pending,cursor,styles)
    }

    fn coordinate_system(&self) -> &CoordinateSystem { &self.coord_system }
    fn locate(&mut self, prep: &mut BoxPositionContext, value: &StaticValue<f64>) {
        let value = cache_constant_clonable(short_memoized_clonable(value.clone()));
        let mut kids = self.kids.children.values_mut().collect::<Vec<_>>();
        let padding_top = self.style.padding.padding_top;
        let draw_top = cache_constant(derived(value.clone(),move |top| top+padding_top)).derc();
        self.top_setter.set(value.clone());
        if let Some(datum) = &self.style.set_align {
            prep.state_request.aligner_mut().set(datum,value);
        }
        self.specifics.set_locate(prep,&draw_top,&mut kids);
    }

    fn name(&self) -> &AllotmentName { &self.name }

    fn build(&mut self, prep: &mut BoxPositionContext) -> BuildSize {
        let mut ranges = vec![];
        let mut input = vec![];
        for child in self.kids.children.values_mut() {
            let size = child.build(prep);
            ranges.push(size.range.clone());
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
        let range = ranges.iter().fold(RangeUsed::None,|a,b| { a.merge(b) });
        BuildSize {
            name: self.name.clone(),
            height,
            range
        }
    }

    fn priority(&self) -> i64 { self.priority }
}