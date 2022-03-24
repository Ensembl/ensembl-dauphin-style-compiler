use std::sync::{ Arc, Mutex };
use peregrine_toolkit::lock;

use crate::{PlayingField, TrainState};
use crate::allotment::core::allotmentmetadata::AllotmentMetadataReport;
use crate::api::{ CarriageSpeed, PeregrineCore };
use crate::train::Carriage;
use crate::train::train::Train;
use crate::core::Viewport;

use super::carriage::DrawingCarriage;

enum RailwayEvent {
    DrawSendAllotmentMetadata(AllotmentMetadataReport),
    LoadTrainData(Train),
    LoadCarriageData(Carriage),
    DrawSetCarriages(Train,Vec<Carriage>),
    DrawStartTransition(Train,u64,CarriageSpeed),
    DrawNotifyViewport(Viewport,bool),
    DrawNotifyPlayingField(PlayingField),
    DrawCreateTrain(Train),
    DrawDropTrain(Train),
    DrawDropCarriage(Carriage)
}

#[derive(Clone)]
pub(super) struct RailwayEvents(Arc<Mutex<Vec<RailwayEvent>>>);

impl RailwayEvents {
    pub(super) fn new() -> RailwayEvents {
        RailwayEvents(Arc::new(Mutex::new(vec![])))
    }

    pub fn len(&self) -> usize { lock!(self.0).len() }

    pub(super) fn load_train_data(&mut self, train: &Train) {
        self.0.lock().unwrap().push(RailwayEvent::LoadTrainData(train.clone()));
    }

    pub(super) fn draw_send_allotment_metadata(&mut self, metadata: &AllotmentMetadataReport) {
        self.0.lock().unwrap().push(RailwayEvent::DrawSendAllotmentMetadata(metadata.clone()));
    }

    pub(super) fn load_carriage_data(&mut self, carriage: &Carriage) {
        self.0.lock().unwrap().push(RailwayEvent::LoadCarriageData(carriage.clone()));
    }

    pub(super) fn draw_set_carriages(&mut self, train: &Train, carriages: &[Carriage]) {
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

    pub(super) fn draw_drop_carriage(&mut self, carriage: &Carriage) {
        self.0.lock().unwrap().push(RailwayEvent::DrawDropCarriage(carriage.clone()));
    }

    pub(super) fn run_events(&mut self, objects: &mut PeregrineCore) -> Vec<Carriage> {
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
                    let drawing_carriages = carriages.iter().map(|c| DrawingCarriage::new(c,&TrainState::independent())).collect::<Vec<_>>();
                    let r = lock!(objects.base.integration).set_carriages(&train,&drawing_carriages);
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
                RailwayEvent::DrawDropCarriage(carriage) => {
                    let drawing_carriage = DrawingCarriage::new(&carriage,&TrainState::independent());
                    lock!(objects.base.integration).drop_carriage(&drawing_carriage);
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
