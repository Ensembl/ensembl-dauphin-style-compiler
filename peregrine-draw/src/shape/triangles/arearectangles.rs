use std::sync::{Arc, Mutex};
use peregrine_data::{AuxLeaf, SpaceBaseArea, SpaceBase, HollowEdge2, reactive::{Observable, Observer}, PartialSpaceBase};
use peregrine_toolkit::{eachorevery::EachOrEvery, lock, error::Error};
use super::rectangles::{RectanglesImpl, apply_wobble};

fn apply_hollow(area: &SpaceBaseArea<f64,AuxLeaf>, edge: &Option<HollowEdge2<f64>>) -> SpaceBaseArea<f64,AuxLeaf> {
    if let Some(edge) = edge {
        area.hollow_edge(&edge)
    } else {
        area.clone()
    }
}

fn area_to_rectangle(area: &SpaceBaseArea<f64,AuxLeaf>,  wobble: &Option<SpaceBaseArea<Observable<'static,f64>,()>>, edge: &Option<HollowEdge2<f64>>) -> Result<SpaceBaseArea<f64,AuxLeaf>,Error> {
    if let Some(wobble) = wobble {
        let top_left = apply_wobble(area.top_left(),wobble.top_left());
        let bottom_right = apply_wobble(area.bottom_right(),wobble.bottom_right());
        let wobbled = SpaceBaseArea::new(PartialSpaceBase::from_spacebase(top_left),PartialSpaceBase::from_spacebase(bottom_right));
        if let Some(wobbled) = wobbled {
            return Ok(apply_hollow(&wobbled,edge));
        }
    }
    Ok(apply_hollow(area,edge))
}


#[cfg_attr(debug_assertions,derive(Debug))]
pub(super) struct RectanglesLocationArea {
    spacebase: SpaceBaseArea<f64,AuxLeaf>,
    wobble: Option<SpaceBaseArea<Observable<'static,f64>,()>>,
    wobbled_spacebase: Arc<Mutex<SpaceBaseArea<f64,AuxLeaf>>>,
    depth: EachOrEvery<i8>,
    edge: Option<HollowEdge2<f64>>
}

impl RectanglesLocationArea {
    pub(super) fn new(spacebase: &SpaceBaseArea<f64,AuxLeaf>, wobble: Option<SpaceBaseArea<Observable<'static,f64>,()>>, depth: EachOrEvery<i8>, edge: Option<HollowEdge2<f64>>) -> Result<RectanglesLocationArea,Error> {
        Ok(RectanglesLocationArea {
            wobbled_spacebase: Arc::new(Mutex::new(area_to_rectangle(spacebase,&wobble,&edge)?)),
            spacebase: spacebase.clone(),
            wobble, depth, edge,
        })
    }
}

impl RectanglesImpl for RectanglesLocationArea {
    fn depths(&self) -> &EachOrEvery<i8> { &self.depth }
    fn any_dynamic(&self) -> bool { self.wobble.is_some() }
    fn len(&self) -> usize { self.spacebase.len() }

    fn wobbled_location(&self) -> (SpaceBaseArea<f64,AuxLeaf>,Option<SpaceBase<f64,()>>) {
        (lock!(self.wobbled_spacebase).clone(),None)
    }

    fn wobble(&mut self) -> Option<Box<dyn FnMut() + 'static>> {
        self.wobble.as_ref().map(|wobble| {
            let wobble = wobble.clone();
            let area = self.spacebase.clone();
            let wobbled = self.wobbled_spacebase.clone();
            let edge = self.edge.clone();
            Box::new(move || {
                if let Ok(area) = area_to_rectangle(&area,&Some(wobble.clone()),&edge) {
                    *lock!(wobbled) = area;
                }
            }) as Box<dyn FnMut() + 'static>
        })
    }

    fn watch(&self, observer: &mut Observer<'static>) {
        if let Some(wobble) = &self.wobble {
            for obs in wobble.top_left().iter() {
                observer.observe(obs.base);
                observer.observe(obs.normal);
                observer.observe(obs.tangent);
            }
            for obs in wobble.bottom_right().iter() {
                observer.observe(obs.base);
                observer.observe(obs.normal);
                observer.observe(obs.tangent);
            }
        }
    }
}
