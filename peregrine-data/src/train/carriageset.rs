use std::cmp::max;
use std::collections::{HashSet, HashMap};
use std::ops::Range;
use peregrine_toolkit::sync::needed::Needed;

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

enum CarriagePrepState {
    Waiting(CarriageProcess),
    Ready(DrawingCarriage)
}

impl CarriagePrepState {
    fn drawing_carriage(&self) -> Option<&DrawingCarriage> {
        match self {
            CarriagePrepState::Waiting(_) => None,
            CarriagePrepState::Ready(dc) => Some(dc)
        }
    }
}

pub struct DrawingCarriageSet {
    constant: CarriageSetConstant,
    index: Option<u64>,
    train_state: Option<TrainState>,
    carriage_processes: HashMap<u64,CarriageProcess>,
    drawing_carriages: HashMap<u64,CarriagePrepState>,
    changes_pending: bool 
}

impl DrawingCarriageSet {
    pub(crate) fn new(try_lifecycle: &Needed, extent: &TrainExtent, configs: &TrainTrackConfigList, messages: &MessageSender) -> DrawingCarriageSet {
        let constant = CarriageSetConstant::new(try_lifecycle,extent,configs,messages);
        DrawingCarriageSet {
            constant,
            index: None,
            train_state: None,
            carriage_processes: HashMap::new(),
            drawing_carriages: HashMap::new(),
            changes_pending: false
        }
    }

    fn clear_drawing_carriages(&mut self, railway_events: &mut RailwayEvents) {
        /* old carriages to dispose of either because update doesn't need it or whole class is being dropped */
        for (_,c) in self.drawing_carriages.drain() {
            if let CarriagePrepState::Ready(dc) = &c {
                railway_events.draw_drop_carriage(dc);
            }
        }
    }

    fn remove_unused_carriages(&mut self) {
        let drawing_carriage_indexes = self.drawing_carriages.keys().cloned().collect::<HashSet<_>>();
        let carriage_indexes = self.carriage_processes.keys().cloned().collect::<HashSet<_>>();
        for dead_carriage_index in  carriage_indexes.difference(&drawing_carriage_indexes) {
            self.carriage_processes.remove(dead_carriage_index);
        }
    }

    pub(crate) fn discard(&mut self, railway_events: &mut RailwayEvents) {
        self.clear_drawing_carriages(railway_events);
        self.carriage_processes.clear();
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
                self.clear_drawing_carriages(railway_events);
            }
        }
        self.train_state = Some(train_state.clone());
        self.update_carriages(railway_events);
    }

    fn try_new_carriage_process(&mut self, index: u64, railway_events: &mut RailwayEvents) -> CarriageProcess {
        if let Some(carriage) = self.carriage_processes.get(&index) {
            return carriage.clone();
        }
        let new_carriage = self.constant.new_unloaded_carriage(index);
        self.carriage_processes.insert(index,new_carriage.clone());
        railway_events.load_carriage_data(&new_carriage);
        new_carriage
    }

    fn extract_or_create_drawing_carriage(&mut self, index: u64, railway_events: &mut RailwayEvents) -> CarriagePrepState {
        if let Some(existing) = self.drawing_carriages.remove(&index) {
            /* old carriage to keep */
            existing
        } else {
            /* new carriage */
            let new_carriage = self.try_new_carriage_process(index,railway_events);
            CarriagePrepState::Waiting(new_carriage.clone())
        }
    }

    fn update_carriages(&mut self, railway_events: &mut RailwayEvents) {
        if self.index.is_none() { return; }
        /* Update list of carriages. We populate a new carriage list by draining from the current list where available
         * or by scheduling creation of a new one. Anything left in the list is then discarded. We then replace the old
         * list with our new one.
         */
        let mut new_carriages = HashMap::new();
        for index in self.wanted_carriage_points(&self.constant.extent,self.index.unwrap()) {
            new_carriages.insert(index,self.extract_or_create_drawing_carriage(index,railway_events));
        }
        /* remove any old carriages left */
        self.clear_drawing_carriages(railway_events);
        /* update ourselves to new carriage set */
        self.drawing_carriages = new_carriages;
        self.remove_unused_carriages();
        self.changes_pending = true;
        /* We probably have some already! */
        self.check_for_carriages_with_shapes(railway_events);
    }

    /* Check if anything we are waiting for is now ready.
     */
    pub(super) fn check_for_carriages_with_shapes(&mut self, railway_events: &mut RailwayEvents) {
        for p in self.drawing_carriages.values_mut() {
            if let (CarriagePrepState::Waiting(carriage),Some(train_state)) = (&p,&self.train_state) {
                if let Some(shapes) = carriage.get_shapes() {
                    let drawing_carriage = DrawingCarriage::new(&carriage.extent(),&railway_events,&shapes,&train_state);
                    railway_events.draw_create_carriage(&drawing_carriage);
                    *p = CarriagePrepState::Ready(drawing_carriage);
                    self.changes_pending = true;
                    self.constant.try_lifecycle.set();
                }
            }
        }
    }

    pub(super) fn central_drawing_carriage(&self) -> Option<&DrawingCarriage> {
        self.index
            .and_then(|centre| self.drawing_carriages.get(&centre))
            .and_then(|p| p.drawing_carriage())
    }

    pub(super) fn each_current_drawing_carriage<X,F>(&self, state: &mut X, mut cb: F) where F: FnMut(&mut X,&DrawingCarriage) {
        for p in self.drawing_carriages.values() {
            if let Some(dc) = p.drawing_carriage() {
                cb(state,dc);
            }
        }
    }

    pub(crate) fn all_ready(&self) -> Option<Vec<DrawingCarriage>> {
        let mut out = vec![];
        for dc in self.drawing_carriages.values() {
            if let Some(dc) = dc.drawing_carriage().filter(|x| x.is_ready()) {
                out.push(dc.clone());
            } else {
                return None;
            }
        }
        self.constant.try_lifecycle.set();
        Some(out)
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
