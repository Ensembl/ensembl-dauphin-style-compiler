use std::cmp::max;
use std::collections::{HashSet, HashMap};
use std::iter::FromIterator;
use std::ops::Range;
use peregrine_toolkit::sync::needed::Needed;

use super::carriagelifecycle::CarriageLifecycleSet;
use super::railwayevent::RailwayEvents;
use super::trainextent::TrainExtent;
use crate::allotment::core::heighttracker::HeightTrackerMerger;
use crate::shapeload::carriageprocess::CarriageProcess;
use crate::{CarriageExtent, DrawingCarriage, TrainState, Train};
use crate::api::MessageSender;
use crate::switch::trackconfiglist::TrainTrackConfigList;

const CARRIAGE_FLANK : u64 = 1;
const MILESTONE_CARRIAGE_FLANK : u64 = 1;

#[derive(Clone)]
struct CarriageSetConstant {
    try_lifecycle: Needed,
    extent: TrainExtent,
    configs: TrainTrackConfigList,
    messages: MessageSender
}

impl CarriageSetConstant {
    fn new(try_lifecycle: &Needed, extent: &TrainExtent, configs: &TrainTrackConfigList, messages: &MessageSender) -> CarriageSetConstant {
        CarriageSetConstant {
            try_lifecycle: try_lifecycle.clone(),
            extent: extent.clone(),
            configs: configs.clone(),
            messages: messages.clone()
        }
    }

    fn new_unloaded_carriage(&self, index: u64) -> CarriageProcess {
        CarriageProcess::new(&CarriageExtent::new(&self.extent,index),Some(&self.try_lifecycle),&self.configs,Some(&self.messages),false)
    }
}

struct CarriageProcessSet(HashMap<u64,CarriageProcess>);

impl CarriageProcessSet {
    fn new() -> CarriageProcessSet { CarriageProcessSet(HashMap::new()) }

    fn try_add(&mut self, index: u64, constant: &CarriageSetConstant, railway_events: &mut RailwayEvents) -> CarriageProcess {
        if let Some(carriage) = self.0.get(&index) {
            return carriage.clone();
        }
        let new_carriage = constant.new_unloaded_carriage(index);
        self.0.insert(index,new_carriage.clone());
        railway_events.load_carriage_data(&new_carriage);
        new_carriage
    }

    fn remove_unused(&mut self, used: &mut dyn Iterator<Item=&u64>) {
        let used_indexes = used.cloned().collect::<HashSet<_>>();
        let carriage_indexes = self.0.keys().cloned().collect::<HashSet<_>>();
        for dead_carriage_index in  carriage_indexes.difference(&used_indexes) {
            self.0.remove(dead_carriage_index);
        }
    }
}

pub(super) struct CarriageSet {
    constant: CarriageSetConstant,
    index: Option<u64>,
    train_state: Option<TrainState>,
    carriage_processes: CarriageProcessSet,
    drawing_carriages: CarriageLifecycleSet,
    changes_pending: bool 
}

impl CarriageSet {
    pub(super) fn new(try_lifecycle: &Needed, extent: &TrainExtent, configs: &TrainTrackConfigList, messages: &MessageSender) -> CarriageSet {
        let constant = CarriageSetConstant::new(try_lifecycle,extent,configs,messages);
        CarriageSet {
            constant,
            index: None,
            train_state: None,
            carriage_processes: CarriageProcessSet::new(),
            drawing_carriages: CarriageLifecycleSet::new(),
            changes_pending: false
        }
    }

    pub(super) fn discard(&mut self, railway_events: &mut RailwayEvents) {
        self.drawing_carriages.clear(railway_events);
        self.changes_pending = true;
    }

    /* given our centre, which carriage points do we want? */
    fn wanted_carriage_points(&self, extent: &TrainExtent, centre: u64) -> Range<u64> {
        let flank = if extent.scale().is_milestone() { MILESTONE_CARRIAGE_FLANK } else { CARRIAGE_FLANK };
        let start = max((centre as i64)-(flank as i64),0) as u64;
        start..(start+flank*2+1)
    }

    pub(super) fn update_centre(&mut self, centre: u64, railway_events: &mut RailwayEvents) {
        /* check and update state */
        if let Some(old_centre)= &self.index {
            if *old_centre == centre {
                /* Nothing at all changed */
                return;
            }
        }
        self.index = Some(centre);
        self.update_carriages(railway_events);
    }

    pub(super) fn update_train_state(&mut self, train_state: &TrainState, railway_events: &mut RailwayEvents) {
        /* check and update state */
        if let Some(old_state)= &self.train_state {
            if old_state == train_state {
                /* Nothing at all changed */
                return;
            }
            if old_state != train_state {
                /* If train_state has changed, old DrawingCarriages are no use to us */
                self.drawing_carriages.clear(railway_events);
            }
        }
        self.train_state = Some(train_state.clone());
        self.update_carriages(railway_events);
    }

    fn update_carriages(&mut self, railway_events: &mut RailwayEvents) {
        if self.index.is_none() { return; }
        /* Update list of carriages. We populate a new carriage list by draining from the current list where available
         * or by scheduling creation of a new one. Anything left in the list is then discarded. We then replace the old
         * list with our new one.
         */
        let mut new_set = CarriageLifecycleSet::new();
        for index in self.wanted_carriage_points(&self.constant.extent,self.index.unwrap()) {
            if !self.drawing_carriages.try_transfer(&mut new_set,index) {
                let process = self.carriage_processes.try_add(index,&self.constant,railway_events);
                new_set.add_process(index,process);
            }
        }
        /* remove any old carriages left */
        self.drawing_carriages.clear(railway_events);
        /* update ourselves to new carriage set */
        self.drawing_carriages = new_set;
        self.carriage_processes.remove_unused(&mut self.drawing_carriages.used());
        self.changes_pending = true;
        /* We probably have some already! */
        self.check_for_carriages_with_shapes(railway_events);
    }

    /* Check if anything we are waiting for is now ready.
     */
    pub(super) fn check_for_carriages_with_shapes(&mut self, railway_events: &mut RailwayEvents) {
        if let Some(train_state) = &self.train_state {
            if self.drawing_carriages.try_upgrade(train_state,railway_events) {
                self.changes_pending = true;
                self.constant.try_lifecycle.set();
            }
        }
    }

    pub(super) fn central_drawing_carriage(&self) -> Option<&DrawingCarriage> {
        self.index.and_then(|centre| self.drawing_carriages.get_drawing_carriage(centre))
    }

    pub(super) fn each_current_drawing_carriage<X,F>(&self, state: &mut X, mut cb: F) where F: FnMut(&mut X,&DrawingCarriage) {
        for dc in self.drawing_carriages.each_drawing_carriage() {
            cb(state,dc);
        }
    }

    pub(crate) fn all_ready(&self) -> Option<Vec<DrawingCarriage>> {
        if !self.drawing_carriages.all_ready() { return None; }
        self.constant.try_lifecycle.set();
        Some(Vec::from_iter(self.drawing_carriages.each_drawing_carriage().cloned()))
    }

    pub(super) fn calculate_train_state(&self) -> TrainState {
        let mut merger = HeightTrackerMerger::new();
        self.each_current_drawing_carriage(&mut merger, |merger,carriage| {
            merger.merge(&carriage.intrinsic_height());
        });
        TrainState::new(merger.to_height_tracker())
    }

    pub(super) fn draw_set_carriages(&mut self, train: &Train, railway_events: &mut RailwayEvents) {
        if !self.changes_pending { return; }
        if let Some(carriages) = self.all_ready() {
            railway_events.draw_set_carriages(train,&carriages);    
            self.changes_pending = false;
        }
    }
}
