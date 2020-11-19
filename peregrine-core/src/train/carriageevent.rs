use std::sync::{ Arc, Mutex };
use crate::api::{ CarriageSpeed, PeregrineObjects };
use crate::PgCommanderTaskSpec;
use crate::train::Carriage;

enum CarriageEvent {
    Load(Carriage),
    Set(Vec<Carriage>,u32),
    Transition(u32,CarriageSpeed)
}

#[derive(Clone)]
pub struct CarriageEvents(Arc<Mutex<Vec<CarriageEvent>>>);

impl CarriageEvents {
    pub fn new() -> CarriageEvents {
        CarriageEvents(Arc::new(Mutex::new(vec![])))
    }

    pub fn load(&mut self, carriage: &Carriage) {
        self.0.lock().unwrap().push(CarriageEvent::Load(carriage.clone()));
    }

    pub fn set(&mut self, carriages: &[Carriage], index: u32) {
        self.0.lock().unwrap().push(CarriageEvent::Set(carriages.iter().cloned().collect(),index));
    }

    pub fn transition(&mut self, index: u32, speed: CarriageSpeed) {
        self.0.lock().unwrap().push(CarriageEvent::Transition(index,speed));
    }

    pub fn run(&mut self, objects: &mut PeregrineObjects) {
        let events : Vec<CarriageEvent> = self.0.lock().unwrap().drain(..).collect();
        let mut loads = vec![];
        for e in events {
            match e {
                CarriageEvent::Set(carriages,index) => {
                    objects.integration.lock().unwrap().set_carriages(&carriages,index);
                },
                CarriageEvent::Transition(index,speed) => {
                    objects.integration.lock().unwrap().start_transition(index,speed);
                },
                CarriageEvent::Load(carriage) => {
                    loads.push(carriage);
                }
            }
        }
        objects.train_set.clone().run_load_carriages(objects,loads);
    }
}
