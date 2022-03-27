use std::collections::hash_map::DefaultHasher;
use std::hash::{Hasher, Hash};
use std::sync::{ Arc, Mutex };
use peregrine_toolkit::{lock, log};
use peregrine_toolkit::sync::needed::Needed;

use crate::allotment::core::allotmentmetadata::AllotmentMetadataReport;
use crate::allotment::core::trainstate::TrainState;
use crate::api::{CarriageSpeed, MessageSender, PeregrineCore };
use super::carriageset::{CarriageSet};
use super::railwayevent::RailwayEvents;
use super::trainextent::TrainExtent;
use crate::run::{ add_task, async_complete_task };
use crate::util::message::DataMessage;
use crate::{PgCommanderTaskSpec, DrawingCarriage};
use crate::switch::trackconfiglist::TrainTrackConfigList;
use crate::core::Viewport;

struct TrainData {
    broken: bool,
    active: bool,
    viewport: Viewport,
    max: Option<u64>,
    carriages: CarriageSet,
    messages: MessageSender,
    train_state: TrainState,
    validity_counter: u64
}

impl TrainData {
    fn new(extent: &TrainExtent, try_lifecycle: &Needed, carriage_event: &mut RailwayEvents, viewport: &Viewport, messages: &MessageSender, validity_counter: u64) -> Result<TrainData,DataMessage> {
        let train_track_config_list = TrainTrackConfigList::new(&extent.layout(),&extent.scale());
        let mut out = TrainData {
            broken: false,
            active: false,
            viewport: viewport.clone(),
            carriages: CarriageSet::new(&try_lifecycle, extent,&train_track_config_list,messages),
            max: None,
            messages: messages.clone(),
            train_state: TrainState::independent(),
            validity_counter
        };
        out.set_position(extent,carriage_event,viewport)?;
        Ok(out)
    }

    fn validity_counter(&self) -> u64 { self.validity_counter }

    fn set_active(&mut self, train: &Train, carriage_event: &mut RailwayEvents, speed: CarriageSpeed) {
        self.active = true;
        self.set_drawing_carriages(train,carriage_event);
        carriage_event.draw_start_transition(train,self.max.unwrap(),speed);
    }

    pub(super) fn discard(&mut self, railway_events: &mut RailwayEvents) {
        self.carriages.discard(railway_events);
        self.active = false;
    }

    fn state(&self) -> &TrainState { &self.train_state }
    fn is_active(&self) -> bool { self.active }
    fn viewport(&self) -> &Viewport { &self.viewport }
    fn is_broken(&self) -> bool { self.broken }

    fn allotter_metadata(&self) -> Option<AllotmentMetadataReport> {
        self.carriages.central_drawing_carriage().map(|c| c.solution().metadata())
    }

    fn each_current_drawing_carriage<X,F>(&self, state: &mut X, cb: &F) where F: Fn(&mut X,&DrawingCarriage) {
        self.carriages.each_current_drawing_carriage(state,cb);
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

    fn set_position(&mut self, extent: &TrainExtent, railway_events: &mut RailwayEvents, viewport: &Viewport) -> Result<(),DataMessage> {
        let carriage = extent.scale().carriage(viewport.position()?);
        self.carriages.update_centre(carriage, railway_events);
        self.viewport = viewport.clone();
        Ok(())
    }

    fn train_ready(&self) -> bool { 
        self.carriages.all_ready().is_some()
    }

    fn train_half_ready(&self) -> bool {
        self.carriages.central_drawing_carriage().is_some() && self.max.is_some()
    }

    fn manage_train_state(&mut self) {
        let train_state = self.carriages.calculate_train_state();
        if train_state != self.train_state {
            let mut h = DefaultHasher::new();
            train_state.hash(&mut h);    
            log!("train state changed! {}",h.finish());
            self.train_state = train_state;
        }
    }

    fn set_drawing_carriages(&mut self, train: &Train, events: &mut RailwayEvents) {
        self.carriages.check_for_carriages_with_shapes(events);
        self.manage_train_state();
        self.carriages.update_train_state(&self.train_state, events);
        if self.active {
            self.carriages.draw_set_carriages(train,events);
        }
    }
}

// XXX circular chroms
#[derive(Clone)]
pub struct Train {
    extent: TrainExtent,
    data: Arc<Mutex<TrainData>>
}

impl Train {
    pub(super) fn new(try_lifecycle: &Needed, extent: &TrainExtent, carriage_event: &mut RailwayEvents, viewport: &Viewport, messages: &MessageSender, validity_counter: u64) -> Result<Train,DataMessage> {
        let out = Train {
            data: Arc::new(Mutex::new(TrainData::new(extent,try_lifecycle,carriage_event,viewport,&messages,validity_counter)?)),
            extent: extent.clone()
        };
        carriage_event.load_train_data(&out);
        Ok(out)
    }

    pub(super) fn each_current_drawing_carriage<X,F>(&self, state: &mut X, cb: &F) where F: Fn(&mut X,&DrawingCarriage) {
        lock!(self.data).each_current_drawing_carriage(state,cb);
    }

    pub(super) fn speed_limit(&self, other: &Train) -> CarriageSpeed {
        if self.validity_counter() == other.validity_counter() {
            self.extent().speed_limit(&other.extent())
        } else {
            CarriageSpeed::Slow
        }
    }

    pub fn state(&self) -> TrainState { lock!(self.data).state().clone() }
    pub fn extent(&self) -> TrainExtent { self.extent.clone() }
    pub fn viewport(&self) -> Viewport { self.data.lock().unwrap().viewport().clone() }
    pub fn is_active(&self) -> bool { self.data.lock().unwrap().is_active() }
    pub(super) fn validity_counter(&self) -> u64 { self.data.lock().unwrap().validity_counter() }
    pub(super) fn train_ready(&self) -> bool { self.data.lock().unwrap().train_ready() }
    pub(super) fn train_half_ready(&self) -> bool { self.data.lock().unwrap().train_half_ready() }
    pub(super) fn train_broken(&self) -> bool { self.data.lock().unwrap().is_broken() }
    pub(super) fn allotter_metadata(&self) -> Option<AllotmentMetadataReport> { self.data.lock().unwrap().allotter_metadata() }

    pub(super) fn set_active(&mut self, carriage_event: &mut RailwayEvents, speed: CarriageSpeed) {
        lock!(self.data).set_active(&self.clone(),carriage_event,speed);
    }

    pub(super) fn discard(&mut self, events: &mut RailwayEvents) {
        lock!(self.data).discard(events);
        events.draw_drop_train(self);
    }

    pub(super) fn set_position(&self, carriage_event: &mut RailwayEvents, viewport: &Viewport) -> Result<(),DataMessage> {
        lock!(self.data).set_position(&self.extent,carriage_event,viewport)?;
        Ok(())
    }

    async fn find_max(&self, data: &mut PeregrineCore) -> Result<u64,DataMessage> {
        Ok(data.agent_store.stick_store.get(&self.extent().layout().stick()).await?.size())
    }

    fn set_max(&self, max: Result<u64,DataMessage>) {
        self.data.lock().unwrap().set_max(max);
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

    pub(super) fn set_drawing_carriages(&mut self, events: &mut RailwayEvents) {
        self.data.lock().unwrap().set_drawing_carriages(&self.clone(),events);
    }
}
