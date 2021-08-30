use std::sync::{ Arc, Mutex };
use crate::{AllotmentStaticMetadataBuilder, Scale};
use crate::api::{ CarriageSpeed, PeregrineCore };
use crate::switch::allotment::AllotterMetadata;
use crate::switch::pitch::Pitch;
use crate::train::Carriage;
use crate::train::train::Train;
use crate::core::Viewport;

enum CarriageEvent {
    SendAllotmentMetadata(AllotterMetadata),
    Train(Train),
    Carriage(Carriage),
    Set(Vec<Carriage>,Scale,u32),
    Transition(u32,u64,CarriageSpeed),
    NotifyViewport(Viewport,bool),
    NotifyPitch(Pitch)
}

#[derive(Clone)]
pub(super) struct CarriageEvents(Arc<Mutex<Vec<CarriageEvent>>>);

impl CarriageEvents {
    pub(super) fn new() -> CarriageEvents {
        CarriageEvents(Arc::new(Mutex::new(vec![])))
    }

    pub(super) fn train(&mut self, train: &Train) {
        self.0.lock().unwrap().push(CarriageEvent::Train(train.clone()));
    }

    pub(super) fn send_allotment_metadata(&mut self, metadata: &AllotterMetadata) {
        self.0.lock().unwrap().push(CarriageEvent::SendAllotmentMetadata(metadata.clone()));
    }

    pub(super) fn carriage(&mut self, carriage: &Carriage) {
        self.0.lock().unwrap().push(CarriageEvent::Carriage(carriage.clone()));
    }

    pub(super) fn set_carriages(&mut self, carriages: &[Carriage], scale: Scale, index: u32) {
        self.0.lock().unwrap().push(CarriageEvent::Set(carriages.iter().cloned().collect(),scale,index));
    }

    pub(super) fn transition(&mut self, index: u32, max: u64, speed: CarriageSpeed) {
        self.0.lock().unwrap().push(CarriageEvent::Transition(index,max,speed));
    }

    pub(super) fn notify_viewport(&mut self, viewport: &Viewport, future: bool) {
        self.0.lock().unwrap().push(CarriageEvent::NotifyViewport(viewport.clone(),future));
    }

    pub(super) fn update_pitch(&mut self, pitch: &Pitch) {
        self.0.lock().unwrap().push(CarriageEvent::NotifyPitch(pitch.clone()));
    }

    pub(super) fn run(&mut self, objects: &mut PeregrineCore) -> Vec<Carriage> {
        let events : Vec<CarriageEvent> = self.0.lock().unwrap().drain(..).collect();
        let mut errors = vec![];
        let mut loads = vec![];
        let mut transition = None; /* delay till after corresponding set also eat multiples */
        let mut notifications = vec![];
        for e in events {
            match e {
                CarriageEvent::SendAllotmentMetadata(metadata) => {
                    objects.integration.lock().unwrap().notify_allotment_metadata(&metadata);
                },
                CarriageEvent::Set(carriages,scale,index) => {
                    let r = objects.integration.lock().unwrap().set_carriages(&carriages,scale,index);
                    if let Err(r) = r { errors.push(r); }
                },
                CarriageEvent::Transition(index,max,speed) => {
                    transition = Some((index,max,speed));
                },
                CarriageEvent::Carriage(carriage) => {
                    loads.push(carriage);
                },
                CarriageEvent::Train(train) => {
                    train.run_find_max(objects);
                },
                CarriageEvent::NotifyViewport(viewport, future) => {
                    notifications.push((viewport,future));
                },
                CarriageEvent::NotifyPitch(pitch) => {
                    objects.integration.lock().unwrap().notify_pitch(&pitch);
                }
            }
        }
        if let Some((index,max,speed)) = transition {
            let r = objects.integration.lock().unwrap().start_transition(index,max,speed);
            if let Err(r) = r {
                errors.push(r);
                objects.transition_complete();
            }
        }
        let mut integration =  objects.integration.lock().unwrap();
        for (viewport,future) in notifications.drain(..) {
            integration.notify_viewport(&viewport,future);
        }
        for error in errors.drain(..) {
            objects.base.messages.send(error);
        }
        loads
    }
}
