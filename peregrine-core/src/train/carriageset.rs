use std::cmp::max;
use crate::core::{ PeregrineData };
use super::train::TrainId;
use super::carriage::{ Carriage, CarriageId };

const CARRIAGE_FLANK : u64 = 2;

pub struct CarriageSet {
    carriages: Vec<Carriage>,
    start: u64,
    max: Option<u64>
}

impl CarriageSet {
    fn create(train_id: &TrainId, centre: u64, mut old: CarriageSet) -> (CarriageSet,bool) {
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
            if let Some(max) = old.max {
                if index > max {
                    break;
                }
            }
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
                Carriage::new(&CarriageId::new(train_id,index))
            });
        }
        (CarriageSet { carriages, start, max: old.max },true)
    }

    pub fn new(train_id: &TrainId, centre: u64) -> CarriageSet {
        let fake_old = CarriageSet { carriages: vec![], start: 0, max: None };
        CarriageSet::create(train_id,centre,fake_old).0
    }

    pub fn new_using(train_id: &TrainId, centre: u64, old: CarriageSet) -> (CarriageSet,bool) {
        CarriageSet::create(train_id,centre,old)
    }

    pub async fn load(&self, data: &mut PeregrineData) {
        for carriage in &self.carriages {
            carriage.load(data).await;
        }
    }

    pub fn carriages(&self) -> &Vec<Carriage> { &self.carriages() }

    pub fn set_max(&mut self, max: u64) {
        self.max = Some(max);
    }
}
