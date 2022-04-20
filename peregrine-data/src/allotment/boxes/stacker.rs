use std::sync::Arc;

use peregrine_toolkit::{puzzle::{cache_constant, derived, DelayedSetter, delayed, compose, compose_slice, StaticValue, commute_clonable, cache_constant_clonable }};

use crate::{allotment::{style::{style::{ContainerAllotmentStyle}, allotmentname::{AllotmentNamePart, AllotmentName}}, boxes::boxtraits::Stackable, core::{carriageoutput::BoxPositionContext}}, CoordinateSystem};

use super::{container::{Container}, boxtraits::{Coordinated, BuildSize, ContainerSpecifics}};

#[derive(Clone)]
pub struct Stacker(Container);

impl Stacker {
    pub(crate) fn new(name: &AllotmentNamePart, style: &ContainerAllotmentStyle) -> Stacker {
        Stacker(Container::new(name,style,UnpaddedStacker::new()))
    }

    pub(crate) fn add_child(&mut self, child: &dyn Stackable) {
        self.0.add_child(child)
    }
}

#[derive(Clone)]
struct AddedChild {
    priority: i64,
    height: StaticValue<f64>
}

fn child_tops<'a>(children: &[AddedChild]) -> (StaticValue<Arc<Vec<f64>>>,StaticValue<f64>) {
    let mut children = children.iter().enumerate().collect::<Vec<_>>();
    children.sort_by_cached_key(|c| c.1.priority);
    let positions = Arc::new(children.iter().map(|c| c.0).collect::<Vec<_>>());
    let heights = children.iter().map(|c| c.1.height.clone()).collect::<Vec<_>>();
    /* calculate our own height */
    let self_height = commute_clonable(&heights,0.,|a,b| *a+*b);
    /* collate child heights */
    let heights = compose_slice(&heights,|x| x.to_vec());
    /* set relative tops */
    let relative_tops = cache_constant(derived(heights,move |heights| {
        let mut tops = vec![];
        let mut top = 0.;
        for height in &*heights {
            tops.push(top);
            top += *height;
        }
        let mut out = vec![0.;tops.len()];
        for (i,pos) in positions.iter().enumerate() {
            out[*pos] = tops[i];
        }
        out
    }));
    (relative_tops,self_height)
}

#[derive(Clone)]
struct UnpaddedStacker {
    relative_tops: StaticValue<Option<Arc<Vec<f64>>>>,
    relative_tops_setter: DelayedSetter<'static,'static,Arc<Vec<f64>>>
}

impl UnpaddedStacker {
    fn new() -> UnpaddedStacker {
        let (relative_tops_setter,relative_tops) = delayed();
        UnpaddedStacker {
            relative_tops_setter, relative_tops
        }
    }
}

impl Stackable for Stacker {
    fn cloned(&self) -> Box<dyn Stackable> { Box::new(self.clone()) }
    fn name(&self) -> &AllotmentName { self.0.name( )}
    fn locate(&mut self, prep: &mut BoxPositionContext, top: &StaticValue<f64>) { self.0.locate(prep,top); }
    fn priority(&self) -> i64 { self.0.priority() }
    fn build(&mut self, prep: &mut BoxPositionContext) -> BuildSize { self.0.build(prep) }
}

impl Coordinated for Stacker {
    fn coordinate_system(&self) -> &CoordinateSystem { self.0.coordinate_system() }
}

impl ContainerSpecifics for UnpaddedStacker {
    fn cloned(&self) -> Box<dyn ContainerSpecifics> { Box::new(self.clone()) }

    fn build_reduce(&mut self, _prep: &mut BoxPositionContext, children: &[(&Box<dyn Stackable>,BuildSize)]) -> StaticValue<f64> {
        let mut added = vec![];
        for (child,size) in children {
            added.push(AddedChild {
                height: size.height.clone(),
                priority: child.priority()
            });
        }
        let (relative_tops,self_height) = child_tops(&added);
        self.relative_tops_setter.set(relative_tops);
        self_height
    }

    fn set_locate(&mut self, prep: &mut BoxPositionContext, top: &StaticValue<f64>, children: &mut [&mut Box<dyn Stackable>]) {
        for (i,child) in children.iter_mut().enumerate() {
            let relative_top = derived(self.relative_tops.clone(),move |tops|
                tops.unwrap()[i]
            );
            let abs_top = cache_constant_clonable(compose(top.clone(),relative_top,|a,b| a+b));
            child.locate(prep,&abs_top);
        }
    }
}
