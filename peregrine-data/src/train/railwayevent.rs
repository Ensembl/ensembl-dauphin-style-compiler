use std::sync::{ Arc, Mutex };
use peregrine_toolkit::lock;
use peregrine_toolkit::sync::needed::Needed;

use crate::shapeload::carriageprocess::CarriageProcess;
use crate::{PlayingField};
use crate::allotment::core::allotmentmetadata::AllotmentMetadataReport;
use crate::api::{ CarriageSpeed, PeregrineCore };
use crate::train::train::Train;
use crate::core::Viewport;

use super::carriage::{DrawingCarriage};

enum RailwayEvent {
    DrawSendAllotmentMetadata(AllotmentMetadataReport),
    LoadTrainData(Train),
    LoadCarriageData(CarriageProcess),
    DrawSetCarriages(Train,Vec<DrawingCarriage>),
    DrawStartTransition(Train,u64,CarriageSpeed),
    DrawNotifyViewport(Viewport,bool),
    DrawNotifyPlayingField(PlayingField),
    DrawCreateTrain(Train),
    DrawDropTrain(Train),
    DrawCreateCarriage(DrawingCarriage),
    DrawDropCarriage(DrawingCarriage)
}

#[derive(Clone)]
pub(crate) struct RailwayEvents(Arc<Mutex<Vec<RailwayEvent>>>,Needed);

impl RailwayEvents {
    pub(super) fn new(try_lifecycle: &Needed) -> RailwayEvents {
        RailwayEvents(Arc::new(Mutex::new(vec![])),try_lifecycle.clone())
    }

    pub fn lifecycle(&self) -> &Needed { &self.1 }

    pub fn len(&self) -> usize { lock!(self.0).len() }

    pub(super) fn load_train_data(&mut self, train: &Train) {
        self.0.lock().unwrap().push(RailwayEvent::LoadTrainData(train.clone()));
    }

    pub(super) fn draw_send_allotment_metadata(&mut self, metadata: &AllotmentMetadataReport) {
        self.0.lock().unwrap().push(RailwayEvent::DrawSendAllotmentMetadata(metadata.clone()));
    }

    pub(super) fn load_carriage_data(&mut self, carriage: &CarriageProcess) {
        self.0.lock().unwrap().push(RailwayEvent::LoadCarriageData(carriage.clone()));
    }

    pub(super) fn draw_set_carriages(&mut self, train: &Train, carriages: &[DrawingCarriage]) {
        self.0.lock().unwrap().push(RailwayEvent::DrawSetCarriages(train.clone(),carriages.iter().cloned().collect()));
    }

    pub(super) fn draw_start_transition(&mut self, train: &Train, max: u64, speed: CarriageSpeed) {
        self.0.lock().unwrap().push(RailwayEvent::DrawStartTransition(train.clone(),max,speed));
    }

    pub(super) fn draw_notify_viewport(&mut self, viewport: &Viewport, future: bool) {
        self.0.lock().unwrap().push(RailwayEvent::DrawNotifyViewport(viewport.clone(),future));
    }

    pub(super) fn draw_notify_playingfield(&mut self, playing_field: PlayingField) {
        self.0.lock().unwrap().push(RailwayEvent::DrawNotifyPlayingField(playing_field));
    }

    pub(super) fn draw_create_train(&mut self, train: &Train) {
        self.0.lock().unwrap().push(RailwayEvent::DrawCreateTrain(train.clone()));
    }

    pub(super) fn draw_drop_train(&mut self, train: &Train) {
        self.0.lock().unwrap().push(RailwayEvent::DrawDropTrain(train.clone()));
    }

    pub(super) fn draw_create_carriage(&mut self, carriage: &DrawingCarriage) {
        self.0.lock().unwrap().push(RailwayEvent::DrawCreateCarriage(carriage.clone()));
    }

    pub(super) fn draw_drop_carriage(&mut self, carriage: &DrawingCarriage) {
        self.0.lock().unwrap().push(RailwayEvent::DrawDropCarriage(carriage.clone()));
    }

    pub(super) fn run_events(&mut self, objects: &mut PeregrineCore) -> Vec<CarriageProcess> {
        let events : Vec<RailwayEvent> = self.0.lock().unwrap().drain(..).collect();
        let mut errors = vec![];
        let mut loads = vec![];
        let mut transition = None; /* delay till after corresponding set also eat multiples */
        let mut notifications = vec![];
        for e in events {
            match e {
                RailwayEvent::DrawSendAllotmentMetadata(metadata) => {
                    objects.base.integration.lock().unwrap().notify_allotment_metadata(&metadata);
                },
                RailwayEvent::DrawSetCarriages(train,carriages) => {
                    let r = lock!(objects.base.integration).set_carriages(&train,&carriages);
                    if let Err(r) = r { errors.push(r); }
                },
                RailwayEvent::DrawStartTransition(index,max,speed) => {
                    transition = Some((index,max,speed));
                },
                RailwayEvent::LoadCarriageData(carriage) => {
                    loads.push(carriage);
                },
                RailwayEvent::LoadTrainData(train) => {
                    train.run_find_max(objects);
                },
                RailwayEvent::DrawNotifyViewport(viewport, future) => {
                    notifications.push((viewport,future));
                },
                RailwayEvent::DrawNotifyPlayingField(height) => {
                    lock!(objects.base.integration).set_playing_field(height);
                },
                RailwayEvent::DrawCreateTrain(train) => {
                    lock!(objects.base.integration).create_train(&train);
                },
                RailwayEvent::DrawDropTrain(train) => {
                    lock!(objects.base.integration).drop_train(&train);
                },
                RailwayEvent::DrawCreateCarriage(carriage) => {
                    lock!(objects.base.integration).create_carriage(&carriage);
                }
                RailwayEvent::DrawDropCarriage(carriage) => {
                    lock!(objects.base.integration).drop_carriage(&carriage);
                }
            }
        }
        if let Some((train,max,speed)) = transition {
            let r = objects.base.integration.lock().unwrap().start_transition(&train,max,speed);
            if let Err(r) = r {
                errors.push(r);
                objects.transition_complete();
            }
        }
        let mut integration =  objects.base.integration.lock().unwrap();
        for (viewport,future) in notifications.drain(..) {
            integration.notify_viewport(&viewport,future);
        }
        for error in errors.drain(..) {
            objects.base.messages.send(error);
        }
        loads
    }
}
