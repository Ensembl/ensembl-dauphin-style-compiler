use std::cmp::max;
use super::train::TrainId;
use super::carriageevent::CarriageEvents;
use super::carriage::{ Carriage, CarriageId };
use crate::api::MessageSender;
use peregrine_message::Reporter;
use crate::util::message::DataMessage;

const CARRIAGE_FLANK : u64 = 2;

pub struct CarriageSet {
    carriages: Vec<Carriage>,
    start: u64,
    pending: Option<Reporter<DataMessage>>
}

impl CarriageSet {
    fn create(train_id: &TrainId, carriage_events: &mut CarriageEvents, centre: u64, mut old: CarriageSet, messages: &MessageSender, Reporter: &Reporter<DataMessage>) -> CarriageSet {
        let start = max((centre as i64)-(CARRIAGE_FLANK as i64),0) as u64;
        let old_start = old.start;
        let mut pending = old.pending;
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
                let out = Carriage::new(&CarriageId::new(train_id,index),messages);
                carriage_events.carriage(&out,Reporter);
                pending = Some(Reporter.clone());
                out
            });
        }
        CarriageSet { carriages, start, pending }
    }

    pub(super) fn new(train_id: &TrainId, carriage_events: &mut CarriageEvents, centre: u64, messages: &MessageSender, Reporter: &Reporter<DataMessage>) -> CarriageSet {
        CarriageSet { carriages: vec![], start: 0, pending: None }
    }

    pub(super) fn new_using(train_id: &TrainId, carriage_events: &mut CarriageEvents, centre: u64, old: CarriageSet, messages: &MessageSender, Reporter: &Reporter<DataMessage>) -> CarriageSet {
        CarriageSet::create(train_id,carriage_events,centre,old,messages,Reporter)
    }

    pub(super) fn depend(&mut self) -> Option<Reporter<DataMessage>> {
        self.pending.take()
    }

    pub(super) fn carriages(&self) -> &Vec<Carriage> { &self.carriages }

    pub(super) fn ready(&self) -> bool {
        for c in &self.carriages {
            if !c.ready() { return false; }
        }
        true
    }
}
