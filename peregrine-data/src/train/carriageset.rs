use std::cmp::max;
use peregrine_toolkit::sync::needed::Needed;

use super::railwayevent::RailwayEvents;
use super::carriage::{Carriage, CarriageSerialSource};
use super::trainextent::TrainExtent;
use crate::{CarriageExtent};
use crate::api::MessageSender;
use crate::switch::trackconfiglist::TrainTrackConfigList;

const CARRIAGE_FLANK : u64 = 3;

pub struct CarriageSet {
    carriages: Vec<Carriage>,
    start: u64
}

impl CarriageSet {
    fn create(try_lifecycle: &Needed, serial_source: &CarriageSerialSource, train_id: &TrainExtent, configs: &TrainTrackConfigList, carriage_events: &mut RailwayEvents, centre: u64, mut old: CarriageSet, messages: &MessageSender) -> CarriageSet {
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
                if index < *old_index {
                    break;
                } else if index == *old_index {
                    steal = true;
                    break;
                } else {
                    if let Some((_,old)) = old_carriages.next() {
                        old.set_moribund(carriage_events);
                    }
                }
            }
            carriages.push(if steal {
                old_carriages.next().unwrap().1
            } else {
                let out = Carriage::new(&try_lifecycle,serial_source,&CarriageExtent::new(train_id,index),configs,Some(messages));
                carriage_events.load_carriage_data(&out);
                out
            });
        }
        for (_,old) in old_carriages {
            old.set_moribund(carriage_events);
        }
        CarriageSet { carriages, start }
    }

    pub(super) fn new() -> CarriageSet {
        CarriageSet { carriages: vec![], start: 0 }
    }

    pub(super) fn size(&self) -> usize { self.carriages.len() }

    pub(super) fn new_using(try_lifecycle: &Needed, serial_source: &CarriageSerialSource, train_id: &TrainExtent, configs: &TrainTrackConfigList, carriage_events: &mut RailwayEvents, centre: u64, old: CarriageSet, messages: &MessageSender) -> CarriageSet {
        CarriageSet::create(try_lifecycle,serial_source,train_id,configs,carriage_events,centre,old,messages)
    }

    pub(super) fn carriages(&self) -> &Vec<Carriage> { &self.carriages }

    pub(super) fn ready(&self) -> bool {
        for c in &self.carriages {
            if !c.ready() { return false; }
        }
        true
    }

    pub(super) fn has_shapes(&self) -> bool {
        for c in &self.carriages {
            if !c.has_shapes() { return false; }
        }
        true
    }

    pub(super) fn discard(&self, events: &mut RailwayEvents) {
        for carriage in &self.carriages {
            carriage.set_moribund(events);
        }        
    }

    pub fn xxx(&self) -> String {
        format!("{:?}",self.carriages.iter().map(|x| x.serial()).collect::<Vec<_>>())
    }
}
