use std::sync::Mutex;

use peregrine_toolkit::{lock, plumbing::onchange::MutexOnChange, sync::{blocker::{Blocker, Lockout}, needed::Needed}};

use crate::{CarriageExtent, ShapeStore, PeregrineCoreBase, DrawingCarriage2};

use super::{anticipate::Anticipate, train::Train};

pub struct RailwayDependents {
    anticipate: Anticipate,
    visual_blocker: Blocker,
    #[allow(unused)]
    visual_lockout: Mutex<Option<Lockout>>
}

impl RailwayDependents {
    pub(super) fn new(base: &PeregrineCoreBase, result_store: &ShapeStore, visual_blocker: &Blocker) -> RailwayDependents {
        RailwayDependents {
            anticipate: Anticipate::new(base,result_store),
            visual_blocker: visual_blocker.clone(),
            visual_lockout: Mutex::new(None),
        }
    }

    pub(super) fn position_was_updated(&self, carriage_extent: &CarriageExtent) {
        self.anticipate.anticipate(carriage_extent);
    }

    pub(super) fn busy(&self, busy: bool) {
        let mut lockout = lock!(self.visual_lockout);
        if busy {
            if lockout.is_none() {
                *lockout = Some(self.visual_blocker.lock());
            }
        } else {
            *lockout = None;
        }
    }
}
