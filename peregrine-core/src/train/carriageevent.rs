use std::sync::{ Arc, Mutex };
use crate::api::{ CarriageSpeed, PeregrineObjects };
use crate::PgCommanderTaskSpec;
use crate::train::Carriage;
use crate::train::train::Train;

enum CarriageEvent {
    Train(Train),
    Carriage(Carriage),
    Set(Vec<Carriage>,u32),
    Transition(u32,u64,CarriageSpeed)
}

#[derive(Clone)]
pub struct CarriageEvents(Arc<Mutex<Vec<CarriageEvent>>>);

impl CarriageEvents {
    pub fn new() -> CarriageEvents {
        CarriageEvents(Arc::new(Mutex::new(vec![])))
    }

    pub fn train(&mut self, train: &Train) {
        self.0.lock().unwrap().push(CarriageEvent::Train(train.clone()));
    }

    pub fn carriage(&mut self, carriage: &Carriage) {
        self.0.lock().unwrap().push(CarriageEvent::Carriage(carriage.clone()));
    }

    pub fn set(&mut self, carriages: &[Carriage],index: u32) {
        self.0.lock().unwrap().push(CarriageEvent::Set(carriages.iter().cloned().collect(),index));
    }

    pub fn transition(&mut self, index: u32, max: u64, speed: CarriageSpeed) {
        self.0.lock().unwrap().push(CarriageEvent::Transition(index,max,speed));
    }

    pub fn run(&mut self, objects: &mut PeregrineObjects) {
        let events : Vec<CarriageEvent> = self.0.lock().unwrap().drain(..).collect();
        let mut loads = vec![];
        for e in events {
            match e {
                CarriageEvent::Set(carriages,index) => {
                    objects.integration.lock().unwrap().set_carriages(&carriages,index);
                },
                CarriageEvent::Transition(index,max,speed) => {
                    objects.integration.lock().unwrap().start_transition(index,max,speed);
                },
                CarriageEvent::Carriage(carriage) => {
                    loads.push(carriage);
                },
                CarriageEvent::Train(train) => {
                    train.run_find_max(objects);
                }
            }
        }
        objects.train_set.clone().run_load_carriages(objects,loads);
    }
}
