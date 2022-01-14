use crate::{allotment::{core::basicallotmentspec::BasicAllotmentSpec, lineargroup::secondary::{SecondaryPositionStore}}, CoordinateSystem};

fn trim_suffix(suffix: &str, name: &str) -> Option<String> {
    if let Some(start) = name.rfind(":") {
        if &name[start+1..] == suffix {
            return Some(name[0..start].to_string());
        }
    }
    None
}

#[derive(Clone,PartialEq,Eq,Hash)]
enum MTVariety {
    Track,
    TrackWindow,
    Wallpaper
}

impl MTVariety {
    fn from_suffix(spec: &str) -> (MTVariety,String) {
        if let Some(main) = trim_suffix("wallpaper",&spec) {
            (MTVariety::Wallpaper,main)
        } else if let Some(main) = trim_suffix("window",&spec) {
            (MTVariety::TrackWindow,main)
        } else {
            (MTVariety::Track,spec.to_string())
        }
    }
}

#[derive(Clone,PartialEq,Eq,Hash)]
pub(super) struct MTSpecifier {
    variety: MTVariety,
    base: BasicAllotmentSpec
}

impl MTSpecifier {
    pub(super) fn new(spec: &str) -> MTSpecifier {
        let base = BasicAllotmentSpec::from_spec(&spec);
        let (variety,main) = MTVariety::from_suffix(&base.name());
        let base = base.with_name(&main);
        MTSpecifier { variety, base }
    }

    pub(super) fn base(&self) -> &BasicAllotmentSpec { &self.base }

    pub(super) fn sized(&self) -> bool {
        match self.variety {
            MTVariety::Track => true,
            MTVariety::TrackWindow => false,
            MTVariety::Wallpaper => false
        }
    }

    pub(super) fn coord_system(&self, reverse: bool) -> CoordinateSystem {
        match (&self.variety,reverse) {
            (MTVariety::Track,_)           => CoordinateSystem::Tracking,
            (MTVariety::TrackWindow,false) => CoordinateSystem::TrackingWindow,
            (MTVariety::TrackWindow,true)  => CoordinateSystem::TrackingWindowBottom,
            (MTVariety::Wallpaper,false)   => CoordinateSystem::Window,
            (MTVariety::Wallpaper,true)    => CoordinateSystem::WindowBottom
        }
    }

    pub(super) fn get_secondary(&self, secondary_store: &SecondaryPositionStore) -> Option<i64> {
        match self.variety {
            MTVariety::Track => None,
            MTVariety::TrackWindow => None,
            MTVariety::Wallpaper => {
                self.base.secondary().as_ref().and_then(|s| secondary_store.lookup(s))
            }
        }
    }
}
