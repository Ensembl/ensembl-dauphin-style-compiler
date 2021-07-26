use std::cmp::max;
use super::train::TrainId;
use super::carriageevent::CarriageEvents;
use super::carriage::{ Carriage, CarriageId };
use crate::api::MessageSender;
use crate::util::message::DataMessage;
use crate::switch::trackconfiglist::TrainTrackConfigList;

const CARRIAGE_FLANK : u64 = 2;

pub struct CarriageSet {
    carriages: Vec<Carriage>,
    start: u64
}

impl CarriageSet {
    fn create(train_id: &TrainId, configs: &TrainTrackConfigList, carriage_events: &mut CarriageEvents, centre: u64, mut old: CarriageSet, messages: &MessageSender) -> CarriageSet {
        let start = max((centre as i64)-(CARRIAGE_FLANK as i64),0) as u64;
        let old_start = old.start;
        let mut carriages = vec![];
        let mut old_carriages =
            old.carriages.drain(..).enumerate()
               .map(|(i,c)| (old_start + (i as u64),c)).peekable();
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
                let out = Carriage::new(&CarriageId::new(train_id,index),configs,messages);
                carriage_events.carriage(&out);
                out
            });
        }
        CarriageSet { carriages, start }
    }

    pub(super) fn new() -> CarriageSet {
        CarriageSet { carriages: vec![], start: 0 }
    }

    pub(super) fn new_using(train_id: &TrainId, configs: &TrainTrackConfigList, carriage_events: &mut CarriageEvents, centre: u64, old: CarriageSet, messages: &MessageSender) -> CarriageSet {
        CarriageSet::create(train_id,configs,carriage_events,centre,old,messages)
    }

    pub(super) fn carriages(&self) -> &Vec<Carriage> { &self.carriages }

    pub(super) fn ready(&self) -> bool {
        for c in &self.carriages {
            if !c.ready() { return false; }
        }
        true
    }
}
