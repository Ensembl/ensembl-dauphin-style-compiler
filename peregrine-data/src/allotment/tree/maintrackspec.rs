use crate::{allotment::{core::{basicallotmentspec::BasicAllotmentSpec, arbitrator::{Arbitrator, SymbolicAxis, DelayedValue}}}, CoordinateSystem};

use super::leaftransformer::LeafGeometry;

fn trim_suffix(suffix: &str, name: &str) -> Option<String> {
    if let Some(start) = name.rfind(":") {
        if &name[start+1..] == suffix {
            return Some(name[0..start].to_string());
        }
    }
    None
}

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone,PartialEq,Eq,Hash)]
pub enum MTSection {
    Header,
    Main
}

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone,PartialEq,Eq,Hash)]
enum MTVariety {
    Track,
    TrackWindow,
    Wallpaper
}

impl MTVariety {
    fn from_suffix(spec: &str) -> (MTVariety,String,MTSection) {
        if let Some(main) = trim_suffix("wallpaper",&spec) {
            (MTVariety::Wallpaper,main,MTSection::Main)
        } else if let Some(main) = trim_suffix("window",&spec) {
            (MTVariety::TrackWindow,main,MTSection::Main)
        } else if let Some(main) = trim_suffix("header",&spec) {
            (MTVariety::Wallpaper,main,MTSection::Header)
        } else {
            (MTVariety::Track,spec.to_string(),MTSection::Main)
        }
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone,PartialEq,Eq,Hash)]
pub(super) struct MTSpecifier {
    variety: MTVariety,
    section: MTSection,
    base: BasicAllotmentSpec
}

impl MTSpecifier {
    pub(super) fn new(spec: &str) -> MTSpecifier {
        let base = BasicAllotmentSpec::from_spec(&spec);
        let (variety,main,section) = MTVariety::from_suffix(&base.name());
        let base = base.with_name(&main);
        MTSpecifier { variety, base, section }
    }

    pub(super) fn base(&self) -> &BasicAllotmentSpec { &self.base }
    pub(super) fn section(&self) -> &MTSection { &self.section }

    pub(super) fn sized(&self) -> bool {
        match self.variety {
            MTVariety::Track => true,
            MTVariety::TrackWindow => false,
            MTVariety::Wallpaper => false
        }
    }

    pub(super) fn arbitrator_horiz(&self, arbitrator: &Arbitrator) -> Option<DelayedValue> {
        match self.variety {
            MTVariety::Track => None,
            MTVariety::TrackWindow => None,
            MTVariety::Wallpaper => {
                self.base.arbitrator().as_ref().and_then(|s| arbitrator.lookup_symbolic_delayed(&SymbolicAxis::ScreenHoriz,s).cloned())
            }
        }
    }

    pub(super) fn our_geometry(&self, input: &LeafGeometry) -> LeafGeometry {
        let coord_system = match (&self.variety,input.reverse()) {
            (MTVariety::Track,_)           => CoordinateSystem::Tracking,
            (MTVariety::TrackWindow,false) => CoordinateSystem::TrackingWindow,
            (MTVariety::TrackWindow,true)  => CoordinateSystem::TrackingWindowBottom,
            (MTVariety::Wallpaper,false)   => CoordinateSystem::Window,
            (MTVariety::Wallpaper,true)    => CoordinateSystem::WindowBottom
        };
        input.with_new_coord_system(&coord_system)
    }
}
