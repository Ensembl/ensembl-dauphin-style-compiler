use std::sync::{ Arc, Mutex };
use crate::api::{ CarriageSpeed, PeregrineCore };
use crate::train::Carriage;
use crate::train::train::Train;

#[derive(Debug)]
enum CarriageEvent {
    Train(Train),
    Carriage(Carriage),
    Set(Vec<Carriage>,u32),
    Transition(u32,u64,CarriageSpeed)
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

    pub(super) fn carriage(&mut self, carriage: &Carriage) {
        self.0.lock().unwrap().push(CarriageEvent::Carriage(carriage.clone()));
    }

    pub(super) fn set_carriages(&mut self, carriages: &[Carriage], index: u32) {
        self.0.lock().unwrap().push(CarriageEvent::Set(carriages.iter().cloned().collect(),index));
    }

    pub(super) fn transition(&mut self, index: u32, max: u64, speed: CarriageSpeed) {
        self.0.lock().unwrap().push(CarriageEvent::Transition(index,max,speed));
    }

    pub(super) fn run(&mut self, objects: &mut PeregrineCore) {
        let events : Vec<CarriageEvent> = self.0.lock().unwrap().drain(..).collect();
        let mut loads = vec![];
        let mut transition = None; /* delay till after corresponding set also eat multiples */
        for e in events {
            match e {
                CarriageEvent::Set(carriages,index) => {
                    objects.integration.lock().unwrap().set_carriages(&carriages,index);
                },
                CarriageEvent::Transition(index,max,speed) => {
                    transition = Some((index,max,speed));
                },
                CarriageEvent::Carriage(carriage) => {
                    loads.push(carriage);
                },
                CarriageEvent::Train(train) => {
                    train.run_find_max(objects);
                }
            }
        }
        if loads.len() > 0 {
            objects.train_set.clone().run_load_carriages(objects,loads);
        }
        if let Some((index,max,speed)) = transition {
            objects.integration.lock().unwrap().start_transition(index,max,speed);
        }
    }
}
