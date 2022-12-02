use std::{sync::{Arc}, collections::HashMap};
use peregrine_toolkit::{puzzle::{DelayedSetter, derived, cache_constant, constant, StaticValue, promise_delayed, short_memoized_clonable, cache_constant_clonable, StaticAnswer }, eachorevery::eoestruct::StructTemplate};
use crate::{allotment::{core::{allotmentname::{AllotmentName, AllotmentNamePart}, boxtraits::{ContainerSpecifics, BuildSize, ContainerOrLeaf}, boxpositioncontext::BoxPositionContext}, style::{style::{ContainerAllotmentStyle, ContainerAllotmentType}}, util::rangeused::RangeUsed, globals::allotmentmetadata::LocalAllotmentMetadataBuilder, stylespec::stylegroup::AllStylesForProgram}, shape::metadata::MetadataStyle, CoordinateSystem, LeafRequest};
use super::{leaf::{AnchoredLeaf, FloatingLeaf}, stacker::{Stacker}, overlay::{Overlay}, bumper::{Bumper}};

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

fn new_container2(name: &AllotmentNamePart, style: &ContainerAllotmentStyle) -> Box<dyn ContainerSpecifics + 'static> {
    match &style.allot_type {
        ContainerAllotmentType::Stack => Box::new(Stacker::new()),
        ContainerAllotmentType::Overlay => Box::new(Overlay::new()),
        ContainerAllotmentType::Bumper => Box::new(Bumper::new(&AllotmentName::from_part(name)))
    }
}

fn new_leaf(pending: &LeafRequest, name: &AllotmentNamePart) -> FloatingLeaf {
    let drawing_info = pending.drawing_info(|di| di.clone());
    let child = FloatingLeaf::new(name,&pending.leaf_style(),&drawing_info);
    child
}
pub(super) struct HasKids {
    pub(super) children: HashMap<ChildKeys,Box<dyn ContainerOrLeaf>>,
    child_leafs: HashMap<String,FloatingLeaf>,
}

impl HasKids {
    pub(super) fn new() -> HasKids {
        HasKids {
            children: HashMap::new(),
            child_leafs: HashMap::new(),   
        }
    }

    pub(super) fn get_leaf(&mut self, pending: &LeafRequest, cursor: usize, styles: &Arc<AllStylesForProgram>) -> FloatingLeaf {
        let name = pending.name().sequence();
        let step = &name[cursor];
        if cursor == name.len() - 1 {
            /* leaf */
            if !self.child_leafs.contains_key(step) {
                /* create leaf */
                let name = name[0..(cursor+1)].iter().map(|x| x.to_string()).collect::<Vec<_>>();
                let name = AllotmentNamePart::new(AllotmentName::do_new(name,true));
                let leaf = new_leaf(pending,&name);
                self.child_leafs.insert(step.to_string(),leaf.clone());
                self.children.insert(ChildKeys::Leaf(step.to_string()),Box::new(leaf.clone()));
            }
            self.child_leafs.get(step).unwrap().clone()
        } else {
            /* container */
            let key = ChildKeys::Container(step.to_string());
            if !self.children.contains_key(&key) {
                /* create container */
                let name = name[0..(cursor+1)].iter().map(|x| x.to_string()).collect::<Vec<_>>();
                let name = AllotmentNamePart::new(AllotmentName::do_new(name,true));
                let style = styles.get_container(&name);
                let container = Container::new(&name,style,new_container2(&name,style));
                self.children.insert(key.clone(),Box::new(container));
            }
            self.children.get_mut(&key).unwrap().get_leaf(pending,cursor+1,styles).clone()
        }
    }
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
    style: Arc<ContainerAllotmentStyle>,
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
    pub(crate) fn new(name: &AllotmentNamePart, style: &ContainerAllotmentStyle, specifics: Box<dyn ContainerSpecifics + 'static>) -> Container {
        let (top_setter,top) = promise_delayed();
        Container {
            name: AllotmentName::from_part(name),
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
    
    fn get_leaf(&mut self, pending: &LeafRequest, cursor: usize, styles: &Arc<AllStylesForProgram>) -> FloatingLeaf {
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