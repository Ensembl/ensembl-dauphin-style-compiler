use std::sync::{ Arc, Mutex };
use crate::api::{ CarriageSpeed, PeregrineCore };
use crate::train::Carriage;
use crate::train::train::Train;
use crate::util::message::DataMessage;
use peregrine_message::{ Reporter };

enum CarriageEvent {
    Train(Train,Reporter<DataMessage>),
    Carriage(Carriage,Reporter<DataMessage>),
    Set(Vec<Carriage>,u32,Reporter<DataMessage>),
    Transition(u32,u64,CarriageSpeed,Reporter<DataMessage>)
}

#[derive(Clone)]
pub(super) struct CarriageEvents(Arc<Mutex<Vec<CarriageEvent>>>);

impl CarriageEvents {
    pub(super) fn new() -> CarriageEvents {
        CarriageEvents(Arc::new(Mutex::new(vec![])))
    }

    pub(super) fn train(&mut self, train: &Train, reporter: &Reporter<DataMessage>) {
        self.0.lock().unwrap().push(CarriageEvent::Train(train.clone(),reporter.clone()));
    }

    pub(super) fn carriage(&mut self, carriage: &Carriage, reporter: &Reporter<DataMessage>) {
        self.0.lock().unwrap().push(CarriageEvent::Carriage(carriage.clone(),reporter.clone()));
    }

    pub(super) fn set_carriages(&mut self, carriages: &[Carriage], index: u32, reporter: &Reporter<DataMessage>) {
        self.0.lock().unwrap().push(CarriageEvent::Set(carriages.iter().cloned().collect(),index,reporter.clone()));
    }

    pub(super) fn transition(&mut self, index: u32, max: u64, speed: CarriageSpeed, reporter: &Reporter<DataMessage>) {
        self.0.lock().unwrap().push(CarriageEvent::Transition(index,max,speed,reporter.clone()));
    }

    pub(super) fn run(&mut self, objects: &mut PeregrineCore) -> Vec<(Carriage,Reporter<DataMessage>)> {
        let events : Vec<CarriageEvent> = self.0.lock().unwrap().drain(..).collect();
        let mut errors = vec![];
        let mut loads = vec![];
        let mut transition = None; /* delay till after corresponding set also eat multiples */
        for e in events {
            match e {
                CarriageEvent::Set(carriages,index,reporter) => {
                    let r = objects.integration.lock().unwrap().set_carriages(&carriages,index);
                    if let Err(r) = r { errors.push((r,reporter)); }
                },
                CarriageEvent::Transition(index,max,speed,reporter) => {
                    transition = Some((index,max,speed,reporter));
                },
                CarriageEvent::Carriage(carriage,reporter) => {
                    loads.push((carriage,reporter));
                },
                CarriageEvent::Train(train, reporter) => {
                    train.run_find_max(objects,&reporter);
                }
            }
        }
        if let Some((index,max,speed,reporter)) = transition {
            let r = objects.integration.lock().unwrap().start_transition(index,max,speed);
            if let Err(r) = r {
                errors.push((r,reporter));
                objects.transition_complete();
            }
        }
        for (error,reporter) in errors.drain(..) {
            reporter.error(error.clone());
            objects.base.messages.send(error);
        }
        loads
    }
}
