use std::sync::{ Arc, Mutex };
use crate::allotment::allotmentmetadata::AllotmentMetadataReport;
use crate::{Scale};
use crate::api::{ CarriageSpeed, PeregrineCore };
use crate::api::PlayingField;
use crate::train::Carriage;
use crate::train::train::Train;
use crate::core::Viewport;

enum RailwayEvent {
    DrawSendAllotmentMetadata(AllotmentMetadataReport),
    LoadTrainData(Train),
    LoadCarriageData(Carriage),
    DrawSetCarriages(Vec<Carriage>,Scale,u32),
    DrawStartTransition(u32,u64,CarriageSpeed),
    DrawNotifyViewport(Viewport,bool),
    DrawNotifyPlayingField(PlayingField)
}

#[derive(Clone)]
pub(super) struct RailwayEvents(Arc<Mutex<Vec<RailwayEvent>>>);

impl RailwayEvents {
    pub(super) fn new() -> RailwayEvents {
        RailwayEvents(Arc::new(Mutex::new(vec![])))
    }

    pub(super) fn load_train_data(&mut self, train: &Train) {
        self.0.lock().unwrap().push(RailwayEvent::LoadTrainData(train.clone()));
    }

    pub(super) fn draw_send_allotment_metadata(&mut self, metadata: &AllotmentMetadataReport) {
        self.0.lock().unwrap().push(RailwayEvent::DrawSendAllotmentMetadata(metadata.clone()));
    }

    pub(super) fn load_carriage_data(&mut self, carriage: &Carriage) {
        self.0.lock().unwrap().push(RailwayEvent::LoadCarriageData(carriage.clone()));
    }

    pub(super) fn draw_set_carriages(&mut self, carriages: &[Carriage], scale: Scale, index: u32) {
        self.0.lock().unwrap().push(RailwayEvent::DrawSetCarriages(carriages.iter().cloned().collect(),scale,index));
    }

    pub(super) fn draw_start_transition(&mut self, index: u32, max: u64, speed: CarriageSpeed) {
        self.0.lock().unwrap().push(RailwayEvent::DrawStartTransition(index,max,speed));
    }

    pub(super) fn draw_notify_viewport(&mut self, viewport: &Viewport, future: bool) {
        self.0.lock().unwrap().push(RailwayEvent::DrawNotifyViewport(viewport.clone(),future));
    }

    pub(super) fn draw_notify_playingfield(&mut self, playing_field: PlayingField) {
        self.0.lock().unwrap().push(RailwayEvent::DrawNotifyPlayingField(playing_field));
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
                RailwayEvent::DrawSetCarriages(carriages,scale,index) => {
                    let r = objects.base.integration.lock().unwrap().set_carriages(&carriages,scale,index);
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
                    objects.base.integration.lock().unwrap().set_playing_field(height);
                }
            }
        }
        if let Some((index,max,speed)) = transition {
            let r = objects.base.integration.lock().unwrap().start_transition(index,max,speed);
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
