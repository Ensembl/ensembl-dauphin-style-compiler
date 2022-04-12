use std::sync::{ Arc, Mutex };
use peregrine_toolkit::{lock};
use peregrine_toolkit::sync::needed::Needed;
use crate::allotment::core::allotmentmetadata::AllotmentMetadataReport;
use crate::allotment::core::trainstate::{TrainState, TrainStateBuilder, TrainState2};
use crate::api::{CarriageSpeed, MessageSender };
use super::railwaydatatasks::RailwayDataTasks;
use super::carriageset::{CarriageSet};
use super::railwayevent::RailwayEvents;
use super::trainextent::TrainExtent;
use crate::util::message::DataMessage;
use crate::{ DrawingCarriage};
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

// XXX circular chroms
pub(super) struct Train {
    extent: TrainExtent,
    active: bool,
    max: Arc<Mutex<StickData>>,
    viewport: Viewport,
    train_state_builder: TrainStateBuilder,
    train_state2: TrainState2,
    train_state: TrainState,
    carriages: CarriageSet,
    validity_counter: u64
}

impl Train {
    pub(super) fn new(try_lifecycle: &Needed, extent: &TrainExtent, carriage_event: &mut RailwayEvents, carriage_loader: &RailwayDataTasks, viewport: &Viewport, messages: &MessageSender, validity_counter: u64) -> Result<Train,DataMessage> {
        let train_track_config_list = TrainTrackConfigList::new(&extent.layout(),&extent.scale());
        let train_state_builder = TrainStateBuilder::new();
        let train_state2 = train_state_builder.state_if_not(None).unwrap();
        let mut out = Train {
            active: false,
            max: Arc::new(Mutex::new(StickData::Pending)),
            extent: extent.clone(),
            viewport: viewport.clone(),
            train_state: TrainState::independent(),
            train_state_builder, train_state2,
            carriages: CarriageSet::new(&try_lifecycle, extent,&train_track_config_list,messages),
            validity_counter
        };
        out.set_position(carriage_event,carriage_loader,viewport)?;
        carriage_loader.add_stick(&out.extent(),&out.stick_data_holder());
        Ok(out)
    }

    pub(super) fn each_current_drawing_carriage<X,F>(&self, state: &mut X, cb: &F) where F: Fn(&mut X,&DrawingCarriage) {
        self.carriages.each_current_drawing_carriage(state,cb);
    }

    pub(super) fn speed_limit(&self, other: &Train) -> CarriageSpeed {
        if self.validity_counter() == other.validity_counter() {
            self.extent().speed_limit(&other.extent())
        } else {
            CarriageSpeed::Slow
        }
    }

    pub(super) fn extent(&self) -> &TrainExtent { &self.extent }
    pub(super) fn viewport(&self) -> &Viewport { &self.viewport }
    pub(super) fn is_active(&self) -> bool { self.active }
    pub(super) fn validity_counter(&self) -> u64 { self.validity_counter }
    pub(super) fn train_ready(&self) -> bool { self.carriages.all_ready().is_some() }

    pub(super) fn train_half_ready(&self) -> bool {
        self.carriages.central_drawing_carriage().is_some() && lock!(self.max).is_ready()
    }

    pub(super) fn train_broken(&self) -> bool { lock!(self.max).is_broken() }

    pub(super) fn allotter_metadata(&self) -> Option<AllotmentMetadataReport> {
        self.carriages.central_drawing_carriage().map(|c| c.solution().metadata())
    }

    pub(super) fn set_active(&mut self, carriage_event: &mut RailwayEvents, carriage_loader: &RailwayDataTasks, speed: CarriageSpeed) {
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
        railway_events.draw_drop_train(&self.extent());
    }

    pub(super) fn set_position(&mut self, railway_events: &mut RailwayEvents, carriage_loader: &RailwayDataTasks, viewport: &Viewport) -> Result<(),DataMessage> {
        let centre_carriage_index = self.extent.scale().carriage(viewport.position()?);
        self.carriages.update_centre(centre_carriage_index,railway_events,carriage_loader);
        self.viewport = viewport.clone();
        Ok(())
    }
    
    pub(super) fn set_drawing_carriages(&mut self, events: &mut RailwayEvents, carriage_loader: &RailwayDataTasks) {
        self.carriages.check_for_carriages_with_shapes(events);
        let train_state = self.carriages.calculate_train_state();
        if train_state != self.train_state {
            self.train_state = train_state;
        }
        self.carriages.update_train_state(&self.train_state, events,carriage_loader);
        if self.active {
            self.carriages.draw_set_carriages(&self.extent,events);
        }
    }

    pub(super) fn stick_data_holder(&self) -> &Arc<Mutex<StickData>> { &self.max }
}
