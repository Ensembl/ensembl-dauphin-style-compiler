use std::{sync::Arc, rc::Rc};
use peregrine_toolkit::puzzle::{ derived, DelayedSetter, delayed, compose, StaticValue, commute_clonable, cache_constant_clonable, compose_slice_vec, short_memoized, cache_constant_rc };
use crate::allotment::layout::{layouttree::ContainerOrLeaf, layoutcontext::LayoutContext, contentsize::ContentSize};

use super::container::ContainerSpecifics;

#[derive(Clone)]
struct AddedChild {
    priority: i64,
    height: StaticValue<f64>
}

fn child_tops<'a>(children: &[AddedChild]) -> (StaticValue<Rc<Vec<f64>>>,StaticValue<f64>) {
    let mut children = children.iter().enumerate().collect::<Vec<_>>();
    children.sort_by_cached_key(|c| c.1.priority);
    let positions = Arc::new(children.iter().map(|c| c.0).collect::<Vec<_>>());
    let heights = children.iter().map(|c| c.1.height.clone()).collect::<Vec<_>>();
    /* calculate our own height */
    let self_height = commute_clonable(&heights,0.,|a,b| *a+*b);
    /* collate child heights */
    let heights = compose_slice_vec(&heights);
    /* set relative tops */
    let relative_tops = cache_constant_rc(short_memoized(derived(heights,move |heights| {
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
    })));
    (relative_tops,self_height)
}

#[derive(Clone)]
pub(super) struct Stacker {
    relative_tops: StaticValue<Option<Rc<Vec<f64>>>>,
    relative_tops_setter: DelayedSetter<'static,'static,Rc<Vec<f64>>>,
}

impl Stacker {
    pub(super) fn new() -> Stacker {
        let (relative_tops_setter,relative_tops) = delayed();
        Stacker {
            relative_tops_setter, relative_tops,
        }
    }
}

impl ContainerSpecifics for Stacker {
    fn build_reduce(&self, _prep: &mut LayoutContext, children: &[(&Box<dyn ContainerOrLeaf>,ContentSize)]) -> StaticValue<f64> {
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

    fn set_locate(&self, prep: &mut LayoutContext, top: &StaticValue<f64>, children: &mut [&mut Box<dyn ContainerOrLeaf>]) {
        for (i,child) in children.iter_mut().enumerate() {
            let relative_top = derived(self.relative_tops.clone(),move |tops|
                tops.unwrap()[i]
            );
            let abs_top = cache_constant_clonable(compose(top.clone(),relative_top,|a,b| a+b));
            child.locate(prep,&abs_top);
        }
    }
}
