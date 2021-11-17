use std::sync::{ Arc, Mutex };
use peregrine_toolkit::lock;

use crate::allotment::allotmentmetadata::AllotmentMetadataReport;
use crate::api::{CarriageSpeed, MessageSender, PeregrineCore };
use super::carriage::Carriage;
use super::carriageset::CarriageSet;
use super::railwayevent::RailwayEvents;
use super::trainextent::TrainExtent;
use crate::run::{ add_task, async_complete_task };
use crate::util::message::DataMessage;
use crate::{PgCommanderTaskSpec};
use crate::switch::trackconfiglist::TrainTrackConfigList;
use crate::core::Viewport;

struct TrainData {
    broken: bool,
    data_ready: bool,
    active: Option<u32>,
    extent: TrainExtent,
    viewport: Viewport,
    max: Option<u64>,
    carriages: Option<CarriageSet>,
    messages: MessageSender,
    track_configs: TrainTrackConfigList
}

impl TrainData {
    fn new(extent: &TrainExtent, carriage_event: &mut RailwayEvents, viewport: &Viewport, messages: &MessageSender) -> Result<TrainData,DataMessage> {
        let train_track_config_list = TrainTrackConfigList::new(&extent.layout(),&extent.scale());
        let mut out = TrainData {
            broken: false,
            data_ready: false,
            active: None,
            extent: extent.clone(),
            viewport: viewport.clone(),
            carriages: Some(CarriageSet::new()),
            max: None,
            messages: messages.clone(),
            track_configs: train_track_config_list,
        };
        out.set_position(carriage_event,viewport)?;
        Ok(out)
    }

    fn set_active(&mut self, train: &Train, carriage_event: &mut RailwayEvents, index: u32, speed: CarriageSpeed) {
        if self.active != Some(index) {
            self.active = Some(index);
            self.set_carriages(train,carriage_event);
            carriage_event.draw_start_transition(train,self.max.unwrap(),speed);
        }
    }

    fn set_inactive(&mut self) { self.active = None; }
    fn is_active(&self) -> bool { self.active.is_some() }
    fn viewport(&self) -> &Viewport { &self.viewport }
    fn extent(&self) -> &TrainExtent { &self.extent }
    fn train_ready(&self) -> bool { self.data_ready && self.max.is_some() }
    fn is_broken(&self) -> bool { self.broken }

    fn central_carriage(&self) -> Option<&Carriage> {
        if let Some(carriages) = &self.carriages {
            let carriages = carriages.carriages();
            if carriages.len() > 0 {
                let central = &carriages[carriages.len()/2];
                if central.ready() {
                    return Some(central);
                }
            }
        }
        None
    }

    fn allotter_metadata(&self) -> Option<AllotmentMetadataReport> {
        self.central_carriage().map(|c| c.shapes().universe().make_metadata_report().clone())
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
        let carriages = CarriageSet::new_using(&self.extent,&self.track_configs,carriage_event,carriage,self.carriages.take().unwrap(),&self.messages);
        self.carriages = Some(carriages);
        Ok(())
    }

    fn check_if_ready(&mut self) {
        if let Some(carriages) = &self.carriages {
            if carriages.ready() && self.max.is_some() {
                self.data_ready = true;
            }
        }
    }

    fn carriages(&self) -> Vec<Carriage> {
        self.carriages.as_ref().map(|x| x.carriages().to_vec()).unwrap_or_else(|| vec![])
    }

    fn set_carriages(&mut self, train: &Train, events: &mut RailwayEvents) {
        if let Some(carriages) = &mut self.carriages {
            if carriages.ready() {
                if let Some(index) = self.active {
                    events.draw_set_carriages(train,&self.carriages());
                }
            }
        }
    }
}

// XXX circular chroms
#[derive(Clone)]
pub struct Train(Arc<Mutex<TrainData>>,MessageSender,u64);

impl Train {
    pub(super) fn new(serial: u64, id: &TrainExtent, carriage_event: &mut RailwayEvents, viewport: &Viewport, messages: &MessageSender) -> Result<Train,DataMessage> {
        let out = Train(Arc::new(Mutex::new(TrainData::new(id,carriage_event,viewport,&messages)?)),messages.clone(),serial);
        carriage_event.load_train_data(&out);
        Ok(out)
    }

    pub fn serial(&self) -> u64 { self.2 }

    pub(super) fn each_current_carriage<X,F>(&self, state: &mut X, cb: &F) where F: Fn(&mut X,&Carriage) {
        lock!(self.0).each_current_carriage(state,cb);
    }

    pub fn extent(&self) -> TrainExtent { self.0.lock().unwrap().extent().clone() }
    pub fn viewport(&self) -> Viewport { self.0.lock().unwrap().viewport().clone() }
    pub fn is_active(&self) -> bool { self.0.lock().unwrap().is_active() }
    pub(super) fn train_ready(&self) -> bool { self.0.lock().unwrap().train_ready() }
    pub(super) fn train_broken(&self) -> bool { self.0.lock().unwrap().is_broken() }
    pub(super) fn allotter_metadata(&self) -> Option<AllotmentMetadataReport> { self.0.lock().unwrap().allotter_metadata() }

    pub(super) fn set_active(&mut self, carriage_event: &mut RailwayEvents, index: u32, speed: CarriageSpeed) {
        self.0.lock().unwrap().set_active(&self.clone(),carriage_event,index,speed);
    }

    pub(super) fn set_inactive(&mut self) {
        self.0.lock().unwrap().set_inactive();
    }

    pub(super) fn set_position(&self, carriage_event: &mut RailwayEvents, viewport: &Viewport) -> Result<(),DataMessage> {
        self.0.lock().unwrap().set_position(carriage_event,viewport)?;
        Ok(())
    }

    pub(super) fn check_if_ready(&mut self) { self.0.lock().unwrap().check_if_ready(); }

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
