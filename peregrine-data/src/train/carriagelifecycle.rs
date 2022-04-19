/*
use std::collections::HashMap;

use crate::{DrawingCarriage, shapeload::carriageprocess::CarriageProcess, TrainState, allotment::core::{trainstate::{TrainStateSpec, TrainState3, CarriageTrainStateSpec}, drawingcarriagedata::DrawingCarriageDataStore}, CarriageExtent};

use super::railwayevent::RailwayEvents;

enum CarriagePrepState {
    Waiting(CarriageProcess),
    //Drawable(DrawingCarriageCreator),
    Ready(DrawingCarriage)
}

impl CarriagePrepState {
    fn drawing_carriage(&self) -> Option<&DrawingCarriage> {
        match self {
            CarriagePrepState::Waiting(_) => None,
            CarriagePrepState::Ready(dc) => Some(dc)
        }
    }

    fn try_upgrade(&mut self, train_state: &TrainState, railway_events: &mut RailwayEvents, index: u64, spec: &mut TrainStateSpec) -> bool {
        if let CarriagePrepState::Waiting(carriage) = self {
            if let Some(shapes) = carriage.get_shapes() {
                let drawing_carriage = DrawingCarriage::new(&carriage.extent(),&railway_events,&shapes,&train_state);
                railway_events.draw_create_carriage(&drawing_carriage);
                spec.add(index,shapes.spec());
                *self = CarriagePrepState::Ready(drawing_carriage);
                return true;
            }
        }
        false
    }
}

pub(super) struct CarriageLifecycleSet(HashMap<u64,CarriagePrepState>);

impl CarriageLifecycleSet {
    pub(super) fn new() -> CarriageLifecycleSet { CarriageLifecycleSet(HashMap::new()) }

    pub(super) fn clear(&mut self, railway_events: &mut RailwayEvents) {
        /* old carriages to dispose of either because update doesn't need it or whole class is being dropped */
        for (_,c) in self.0.drain() {
            if let CarriagePrepState::Ready(dc) = &c {
                railway_events.draw_drop_carriage(dc);
            }
        }
    }

    pub(super) fn try_transfer(&mut self, target: &mut CarriageLifecycleSet, index: u64) -> bool {
        if let Some(value) = target.0.remove(&index) {
            self.0.insert(index,value);
            true
        } else {
            false
        }
    }

    pub(super) fn try_upgrade(&mut self, train_state: &TrainState, railway_events: &mut RailwayEvents, spec: &mut TrainStateSpec) -> bool {
        let mut any = false;
        for (id,p) in self.0.iter_mut() {
            if p.try_upgrade(train_state,railway_events,*id,spec) {
                any = true;
            }
        }
        any
    }

    pub(super) fn each_drawing_carriage(&self) -> impl Iterator<Item=&DrawingCarriage> {
        self.0.values()
            .filter_map(|p| p.drawing_carriage())
    }

    pub(super) fn all_ready(&self) -> bool {
        self.0.values().all(|x| x.drawing_carriage().map(|x| x.is_ready()).unwrap_or(false))
    }

    pub(super) fn add_process(&mut self, index: u64, process: CarriageProcess) {
        self.0.insert(index,CarriagePrepState::Waiting(process));
    }

    pub(super) fn get_drawing_carriage(&self, index: u64) -> Option<&DrawingCarriage> { 
        self.0.get(&index).and_then(|p| p.drawing_carriage())
    }

    pub(super) fn used(&self) -> impl Iterator<Item=u64> + '_ { self.0.keys().cloned() }
}
*/