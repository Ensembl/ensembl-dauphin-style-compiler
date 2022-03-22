use std::sync::{ Arc, Mutex };
use peregrine_toolkit::lock;
use peregrine_toolkit::sync::needed::Needed;

use crate::allotment::core::allotmentmetadata::AllotmentMetadataReport;
use crate::api::{CarriageSpeed, MessageSender, PeregrineCore };
use super::carriage::{Carriage, CarriageSerialSource};
use super::carriageset::CarriageSet;
use super::railwayevent::RailwayEvents;
use super::trainextent::TrainExtent;
use crate::run::{ add_task, async_complete_task };
use crate::util::message::DataMessage;
use crate::{PgCommanderTaskSpec};
use crate::switch::trackconfiglist::TrainTrackConfigList;
use crate::core::Viewport;

struct TrainData {
    try_lifecycle: Needed,
    serial_source: CarriageSerialSource,
    broken: bool,
    active: bool,
    extent: TrainExtent,
    viewport: Viewport,
    max: Option<u64>,
    carriages: Option<CarriageSet>,
    messages: MessageSender,
    track_configs: TrainTrackConfigList,
    validity_counter: u64
}

impl TrainData {
    fn new(extent: &TrainExtent, try_lifecycle: &Needed, carriage_event: &mut RailwayEvents, viewport: &Viewport, messages: &MessageSender, serial_source: &CarriageSerialSource, validity_counter: u64) -> Result<TrainData,DataMessage> {
        let train_track_config_list = TrainTrackConfigList::new(&extent.layout(),&extent.scale());
        let mut out = TrainData {
            try_lifecycle: try_lifecycle.clone(),
            serial_source: serial_source.clone(),
            broken: false,
            active: false,
            extent: extent.clone(),
            viewport: viewport.clone(),
            carriages: Some(CarriageSet::new()),
            max: None,
            messages: messages.clone(),
            track_configs: train_track_config_list,
            validity_counter
        };
        out.set_position(carriage_event,viewport)?;
        Ok(out)
    }

    fn validity_counter(&self) -> u64 { self.validity_counter }

    fn set_active(&mut self, train: &Train, carriage_event: &mut RailwayEvents, speed: CarriageSpeed) {
        self.active = true;
        self.set_carriages(train,carriage_event);
        carriage_event.draw_start_transition(train,self.max.unwrap(),speed);
    }

    pub(super) fn discard(&mut self, events: &mut RailwayEvents) {
        if let Some(carriages) = &self.carriages {
            carriages.discard(events);
        }
        self.active = false;
    }

    fn is_active(&self) -> bool { self.active }
    fn viewport(&self) -> &Viewport { &self.viewport }
    fn extent(&self) -> &TrainExtent { &self.extent }
    fn is_broken(&self) -> bool { self.broken }

    fn central_carriage(&self) -> Option<&Carriage> {
        if let Some(carriages) = &self.carriages {
            let carriages = carriages.carriages();
            if carriages.len() > 0 {
                let central = &carriages[carriages.len()/2];
                if central.has_shapes() {
                    return Some(central);
                }
            }
        }
        None
    }

    fn allotter_metadata(&self) -> Result<Option<AllotmentMetadataReport>,DataMessage> {
        self.central_carriage().map(|c| c.metadata()).transpose()
    }

    fn each_current_carriage<X,F>(&self, state: &mut X, cb: &F) where F: Fn(&mut X,&Carriage) {
        if let Some(carriages) = &self.carriages {
            for carriage in carriages.carriages() {
                cb(state,carriage);
            }
        }
    }

    fn set_max(&mut self, max: Result<u64,DataMessage>) {
        match max {
            Ok(max) => { self.max = Some(max); },
            Err(e) => {
                self.messages.send(e.clone());
                self.broken = true;
            }
        }
    }

    // TODO don't always update CarriageSet
    fn set_position(&mut self, carriage_event: &mut RailwayEvents, viewport: &Viewport) -> Result<(),DataMessage> {
        self.viewport = viewport.clone();
        let carriage = self.extent.scale().carriage(viewport.position()?);
        let carriages = CarriageSet::new_using(&self.try_lifecycle,&self.serial_source,&self.extent,&self.track_configs,carriage_event,carriage,self.carriages.take().unwrap(),&self.messages);
        self.carriages = Some(carriages);
        Ok(())
    }

    fn train_ready(&self) -> bool { 
        if !self.train_half_ready() { return false; }
        if let Some(central) = self.central_carriage() {
            if central.ready() { return true; }
        }
        false
    }

    fn train_half_ready(&self) -> bool {
        if self.max.is_none() { return false; }    
        if let Some(carriages) = &self.carriages {
            if carriages.has_shapes() && self.max.is_some() && carriages.size() > 0 {
                return true;
            }
        }
        false
    }

    fn carriages(&self) -> Vec<Carriage> {
        self.carriages.as_ref().map(|x| x.carriages().to_vec()).unwrap_or_else(|| vec![])
    }

    fn set_carriages(&mut self, train: &Train, events: &mut RailwayEvents) {
        if let Some(_) = &mut self.carriages {
            if self.active {
                events.draw_set_carriages(train,&self.carriages());
            }
        }
    }
}

#[derive(Clone,Debug,Copy,PartialEq,Eq,Hash)]
pub struct TrainSerial(u64);

// XXX circular chroms
#[derive(Clone)]
pub struct Train(Arc<Mutex<TrainData>>,MessageSender,TrainSerial);

impl Train {
    pub(super) fn new(try_lifecycle: &Needed, serial: u64, id: &TrainExtent, carriage_event: &mut RailwayEvents, viewport: &Viewport, messages: &MessageSender, serial_source: &CarriageSerialSource, validity_counter: u64) -> Result<Train,DataMessage> {
        let out = Train(Arc::new(Mutex::new(TrainData::new(id,try_lifecycle,carriage_event,viewport,&messages,serial_source,validity_counter)?)),messages.clone(),TrainSerial(serial));
        carriage_event.load_train_data(&out);
        Ok(out)
    }

    pub fn serial(&self) -> TrainSerial { self.2 }

    pub(super) fn each_current_carriage<X,F>(&self, state: &mut X, cb: &F) where F: Fn(&mut X,&Carriage) {
        lock!(self.0).each_current_carriage(state,cb);
    }

    pub(super) fn speed_limit(&self, other: &Train) -> CarriageSpeed {
        if self.validity_counter() == other.validity_counter() {
            self.extent().speed_limit(&other.extent())
        } else {
            CarriageSpeed::Slow
        }
    }

    pub fn extent(&self) -> TrainExtent { self.0.lock().unwrap().extent().clone() }
    pub fn viewport(&self) -> Viewport { self.0.lock().unwrap().viewport().clone() }
    pub fn is_active(&self) -> bool { self.0.lock().unwrap().is_active() }
    pub(super) fn validity_counter(&self) -> u64 { self.0.lock().unwrap().validity_counter() }
    pub(super) fn train_ready(&self) -> bool { self.0.lock().unwrap().train_ready() }
    pub(super) fn train_half_ready(&self) -> bool { self.0.lock().unwrap().train_half_ready() }
    pub(super) fn train_broken(&self) -> bool { self.0.lock().unwrap().is_broken() }
    pub(super) fn allotter_metadata(&self) -> Result<Option<AllotmentMetadataReport>,DataMessage> { self.0.lock().unwrap().allotter_metadata() }

    pub(super) fn set_active(&mut self, carriage_event: &mut RailwayEvents, speed: CarriageSpeed) {
        lock!(self.0).set_active(&self.clone(),carriage_event,speed);
    }

    pub(super) fn discard(&mut self, events: &mut RailwayEvents) {
        lock!(self.0).discard(events);
        events.draw_drop_train(self);
    }

    pub(super) fn set_position(&self, carriage_event: &mut RailwayEvents, viewport: &Viewport) -> Result<(),DataMessage> {
        lock!(self.0).set_position(carriage_event,viewport)?;
        Ok(())
    }

    async fn find_max(&self, data: &mut PeregrineCore) -> Result<u64,DataMessage> {
        Ok(data.agent_store.stick_store.get(&self.extent().layout().stick()).await?.size())
    }

    fn set_max(&self, max: Result<u64,DataMessage>) {
        self.0.lock().unwrap().set_max(max);
    }
    
    pub(super) fn run_find_max(&self, objects: &mut PeregrineCore) {
        let self2 = self.clone();
        let mut objects2 = objects.clone();
        let handle = add_task(&objects.base.commander,PgCommanderTaskSpec {
            name: format!("max finder"),
            prio: 1,
            slot: None,
            timeout: None,
            task: Box::pin(async move {
                let max = self2.find_max(&mut objects2).await;
                if let Err(e) = &max { 
                    objects2.base.messages.send(e.clone());
                }
                self2.set_max(max);
                objects2.train_set.clone().move_and_lifecycle_trains(&mut objects2);
                Ok(())
            }),
            stats: false
        });
        async_complete_task(&objects.base.commander,&objects.base.messages,handle, |e| (e,false));
    }

    pub(super) fn set_carriages(&mut self, events: &mut RailwayEvents) {
        self.0.lock().unwrap().set_carriages(&self.clone(),events);
    }
}
