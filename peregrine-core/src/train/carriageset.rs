use std::cmp::max;
use crate::api::PeregrineObjects;
use super::train::TrainId;
use super::carriageevent::CarriageEvents;
use super::carriage::{ Carriage, CarriageId };

const CARRIAGE_FLANK : u64 = 2;

pub struct CarriageSet {
    carriages: Vec<Carriage>,
    start: u64
}

impl CarriageSet {
    fn create(train_id: &TrainId, carriage_events: &mut CarriageEvents, centre: u64, mut old: CarriageSet) -> (CarriageSet,bool) {
        let start = max((centre as i64)-(CARRIAGE_FLANK as i64),0) as u64;
        let old_start = old.start;
        if start == old_start {
            return (old,false);
        }
        let mut carriages = vec![];
        let mut old_carriages =
            old.carriages.drain(..).enumerate()
               .map(|(i,c)| (old_start + (i as u64) - CARRIAGE_FLANK as u64,c)).peekable();
        for delta in 0..(CARRIAGE_FLANK*2+1) {
            let index = start + delta;
            let mut steal = false;
            while let Some((old_index,_)) = old_carriages.peek() {
                if index == *old_index {
                    steal = true;
                    break;
                } else {
                    old_carriages.next();
                }
            }
            carriages.push(if steal {
                old_carriages.next().unwrap().1
            } else {
                let out = Carriage::new(&CarriageId::new(train_id,index));
                carriage_events.carriage(&out);
                out
            });
        }
        (CarriageSet { carriages, start },true)
    }

    pub fn new(train_id: &TrainId, carriage_events: &mut CarriageEvents, centre: u64) -> CarriageSet {
        let fake_old = CarriageSet { carriages: vec![], start: 0 };
        CarriageSet::create(train_id,carriage_events,centre,fake_old).0
    }

    pub fn new_using(train_id: &TrainId, carriage_events: &mut CarriageEvents, centre: u64, old: CarriageSet) -> (CarriageSet,bool) {
        CarriageSet::create(train_id,carriage_events,centre,old)
    }

    pub fn send_event(&self, carriage_event: &mut CarriageEvents, index: u32) {
        carriage_event.set(&self.carriages(),index);
    }

    pub fn carriages(&self) -> &Vec<Carriage> { &self.carriages }

    pub fn ready(&self) -> bool {
        for c in &self.carriages {
            if !c.ready() { return false; }
        }
        true
    }
}
