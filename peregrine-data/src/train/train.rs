use std::collections::hash_map::DefaultHasher;
use std::hash::{Hasher, Hash};
use std::sync::{ Arc, Mutex };
use peregrine_toolkit::{lock, log};
use peregrine_toolkit::sync::needed::Needed;
use crate::allotment::core::allotmentmetadata::AllotmentMetadataReport;
use crate::allotment::core::trainstate::TrainState;
use crate::api::{CarriageSpeed, MessageSender };
use super::railwaydatatasks::RailwayDataTasks;
use super::carriageset::{CarriageSet};
use super::railwayevent::RailwayEvents;
use super::trainextent::TrainExtent;
use crate::run::{ add_task, async_complete_task };
use crate::util::message::DataMessage;
use crate::{PgCommanderTaskSpec, DrawingCarriage, PeregrineCoreBase, StickStore};
use crate::switch::trackconfiglist::TrainTrackConfigList;
use crate::core::Viewport;

pub(super) enum StickData {
    Pending,
    Ready(u64),
    Unavailable
}

impl StickData {
    fn is_broken(&self) -> bool { match self { StickData::Unavailable => true, _ => false } }
    fn is_ready(&self) -> bool { match self { StickData::Ready(_) => true, _ => false } }
}

struct TrainData {
    extent: TrainExtent,
    active: bool,
    viewport: Viewport,
    max: Arc<Mutex<StickData>>,
    carriages: CarriageSet,
    train_state: TrainState,
    validity_counter: u64
}

impl TrainData {
    fn new(extent: &TrainExtent, try_lifecycle: &Needed, carriage_event: &mut RailwayEvents, carriage_loader: &RailwayDataTasks, viewport: &Viewport, messages: &MessageSender, validity_counter: u64) -> Result<TrainData,DataMessage> {
        let train_track_config_list = TrainTrackConfigList::new(&extent.layout(),&extent.scale());
        let mut out = TrainData {
            extent: extent.clone(),
            active: false,
            viewport: viewport.clone(),
            carriages: CarriageSet::new(&try_lifecycle, extent,&train_track_config_list,messages),
            max: Arc::new(Mutex::new(StickData::Pending)),
            train_state: TrainState::independent(),
            validity_counter
        };
        out.set_position(carriage_event,carriage_loader,viewport)?;
        Ok(out)
    }

    fn validity_counter(&self) -> u64 { self.validity_counter }

    fn set_active(&mut self, carriage_event: &mut RailwayEvents, carriage_loader: &RailwayDataTasks, speed: CarriageSpeed) {
        let max = match &*lock!(self.max) {
            StickData::Ready(max) => *max,
            _ => { panic!("set_active() called on non-ready train") }
        };
        self.active = true;
        self.set_drawing_carriages(carriage_event,carriage_loader);
        carriage_event.draw_start_transition(&self.extent,max,speed);
    }

    pub(super) fn discard(&mut self, railway_events: &mut RailwayEvents) {
        self.carriages.discard(railway_events);
        self.active = false;
    }

    fn is_active(&self) -> bool { self.active }
    fn viewport(&self) -> &Viewport { &self.viewport }
    fn is_broken(&self) -> bool {
        lock!(self.max).is_broken()
    }

    fn allotter_metadata(&self) -> Option<AllotmentMetadataReport> {
        self.carriages.central_drawing_carriage().map(|c| c.solution().metadata())
    }

    fn each_current_drawing_carriage<X,F>(&self, state: &mut X, cb: &F) where F: Fn(&mut X,&DrawingCarriage) {
        self.carriages.each_current_drawing_carriage(state,cb);
    }

    fn set_position(&mut self, railway_events: &mut RailwayEvents, carriage_loader: &RailwayDataTasks, viewport: &Viewport) -> Result<(),DataMessage> {
        let centre_carriage_index = self.extent.scale().carriage(viewport.position()?);
        self.carriages.update_centre(centre_carriage_index,railway_events,carriage_loader);
        self.viewport = viewport.clone();
        Ok(())
    }

    fn train_ready(&self) -> bool { 
        self.carriages.all_ready().is_some()
    }

    fn train_half_ready(&self) -> bool {
        self.carriages.central_drawing_carriage().is_some() && lock!(self.max).is_ready()
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

    fn set_drawing_carriages(&mut self, events: &mut RailwayEvents, carriage_loader: &RailwayDataTasks) {
        self.carriages.check_for_carriages_with_shapes(events);
        self.manage_train_state();
        self.carriages.update_train_state(&self.train_state, events,carriage_loader);
        if self.active {
            self.carriages.draw_set_carriages(&self.extent,events);
        }
    }
}

// XXX circular chroms
#[derive(Clone)]
pub(super) struct Train {
    extent: TrainExtent,
    data: Arc<Mutex<TrainData>>
}

impl Train {
    pub(super) fn new(try_lifecycle: &Needed, extent: &TrainExtent, carriage_event: &mut RailwayEvents, carriage_loader: &RailwayDataTasks, viewport: &Viewport, messages: &MessageSender, validity_counter: u64) -> Result<Train,DataMessage> {
        let out = Train {
            data: Arc::new(Mutex::new(TrainData::new(extent,try_lifecycle,carriage_event,carriage_loader,viewport,&messages,validity_counter)?)),
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

    pub(super) fn extent(&self) -> TrainExtent { self.extent.clone() }
    pub(super) fn viewport(&self) -> Viewport { self.data.lock().unwrap().viewport().clone() }
    pub(super) fn is_active(&self) -> bool { self.data.lock().unwrap().is_active() }
    pub(super) fn validity_counter(&self) -> u64 { self.data.lock().unwrap().validity_counter() }
    pub(super) fn train_ready(&self) -> bool { self.data.lock().unwrap().train_ready() }
    pub(super) fn train_half_ready(&self) -> bool { self.data.lock().unwrap().train_half_ready() }
    pub(super) fn train_broken(&self) -> bool { self.data.lock().unwrap().is_broken() }
    pub(super) fn allotter_metadata(&self) -> Option<AllotmentMetadataReport> { self.data.lock().unwrap().allotter_metadata() }

    pub(super) fn set_active(&mut self, carriage_event: &mut RailwayEvents, carriage_loader: &RailwayDataTasks, speed: CarriageSpeed) {
        lock!(self.data).set_active(carriage_event,carriage_loader,speed);
    }

    pub(super) fn discard(&mut self, events: &mut RailwayEvents) {
        lock!(self.data).discard(events);
        events.draw_drop_train(&self.extent());
    }

    pub(super) fn set_position(&self, carriage_event: &mut RailwayEvents, carriage_loader: &RailwayDataTasks, viewport: &Viewport) -> Result<(),DataMessage> {
        lock!(self.data).set_position(carriage_event,carriage_loader,viewport)?;
        Ok(())
    }

    async fn find_max(&self, stick_store: &StickStore) -> Result<u64,DataMessage> {
        Ok(stick_store.get(&self.extent().layout().stick()).await?.size())
    }
    
    pub(super) fn set_drawing_carriages(&mut self, events: &mut RailwayEvents, carriage_loader: &RailwayDataTasks) {
        self.data.lock().unwrap().set_drawing_carriages(events,carriage_loader);
    }

    pub(super) fn stick_data_holder(&self) -> Arc<Mutex<StickData>> {
        lock!(self.data).max.clone()        
    }
}
