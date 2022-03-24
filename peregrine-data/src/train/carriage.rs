use std::collections::hash_map::DefaultHasher;
use std::hash::{self, Hash, Hasher};
use std::sync::{ Arc, Mutex };
use peregrine_toolkit::{lock, error};
use peregrine_toolkit::sync::needed::Needed;

use crate::allotment::core::allotmentmetadata::AllotmentMetadataReport;
use crate::allotment::core::carriageuniverse::{CarriageUniverse, CarriageShapes};
use crate::allotment::core::heighttracker::HeightTracker;
use crate::allotment::core::trainstate::TrainState;
use crate::allotment::style::style::LeafCommonStyle;
use crate::api::MessageSender;
use crate::{CarriageExtent, ShapeStore, PeregrineCoreBase, Shape, PlayingField};
use crate::shapeload::{ ShapeRequestGroup };
use crate::util::message::DataMessage;
use crate::switch::trackconfiglist::TrainTrackConfigList;
use crate::shapeload::loadshapes::{LoadMode, load_carriage_shape_list };

use super::railwayevent::RailwayEvents;
use lazy_static::lazy_static;
use identitynumber::identitynumber;

#[derive(Clone)]
struct UnloadedCarriage {
    config: TrainTrackConfigList,
    messages: Option<MessageSender>,
    warm: bool
}

impl UnloadedCarriage {
    fn make_shape_requests(&self, extent: &CarriageExtent) -> ShapeRequestGroup {
        let track_config_list = extent.train().layout().track_config_list();
        let track_list = self.config.list_tracks();
        let pixel_size = extent.train().pixel_size();
        let mut track_configs = vec![];
        for track in track_list {
            if let Some(track_config) = track_config_list.get_track(&track) {
                track_configs.push(track_config.as_ref().clone());
            }
        }
        ShapeRequestGroup::new(&extent.region(),&track_configs,pixel_size,self.warm)
    }

    async fn load(&mut self, extent: &CarriageExtent, base: &PeregrineCoreBase, result_store: &ShapeStore, mode: LoadMode) -> Result<Option<CarriageUniverse>,DataMessage> {
        let shape_requests = self.make_shape_requests(extent);
        let (shapes,errors) = load_carriage_shape_list(base,result_store,self.messages.as_ref(),shape_requests,&mode).await;
        let shapes = if let Some(x) = shapes { x } else { return Ok(None); };
        if errors.len() != 0 {
            error!("{:?}",errors);
            return Err(DataMessage::CarriageUnavailable(extent.clone(),errors));
        }    
        Ok(Some(shapes))
    }
}

enum CarriageState {
    Unloaded(UnloadedCarriage),
    Loading,
    Pending(CarriageShapes),
    Loaded(CarriageShapes)
}

identitynumber!(IDS);

#[derive(Clone)]
pub struct Carriage {
    try_lifecycle: Needed,
    moribund: Arc<Mutex<bool>>,
    serial: u64,
    extent: CarriageExtent,
    state: Arc<Mutex<CarriageState>>
}

impl Carriage {
    pub(crate) fn new(try_lifecycle: &Needed, extent: &CarriageExtent, configs: &TrainTrackConfigList, messages: Option<&MessageSender>, warm: bool) -> Carriage {
        Carriage {
            try_lifecycle: try_lifecycle.clone(),
            moribund: Arc::new(Mutex::new(false)),
            serial: IDS.next(),
            extent: extent.clone(),
            state: Arc::new(Mutex::new(CarriageState::Unloaded(UnloadedCarriage {
                config: configs.clone(),
                messages: messages.cloned(),
                warm
            })))
        }
    }

    fn hash_by_serial<H>(&self, state: &mut H) where H: hash::Hasher {
        self.serial.hash(state);
    }

    pub(crate) fn is_moribund(&self) -> bool { *lock!(self.moribund) }
    pub(crate) fn extent(&self) -> &CarriageExtent { &self.extent }

    pub(super) fn set_moribund(&self,carriage_events: &mut RailwayEvents) {
        *lock!(self.moribund) = true;
        match &*lock!(self.state) {
            CarriageState::Loaded(_) => {
                carriage_events.draw_drop_carriage(self);
            },
            _ => {}
        }
    }

    pub(crate) fn playing_field(&self, train_state: &TrainState) -> PlayingField {
        match &*lock!(self.state) {
            CarriageState::Pending(s) | CarriageState::Loaded(s) => {
                s.get(train_state).playing_field()
            },
            _ => PlayingField::empty()
        }        
    }

    pub(crate) fn metadata(&self, train_state: &TrainState) -> AllotmentMetadataReport {
        match &*lock!(self.state) {
            CarriageState::Pending(s) | CarriageState::Loaded(s) => {
                s.get(train_state).metadata()
            },
            _ => AllotmentMetadataReport::empty()
        }
    }

    fn shapes(&self, train_state: &TrainState) -> Option<Arc<Vec<Shape<LeafCommonStyle>>>> {
        match &*lock!(self.state) {
            CarriageState::Pending(s) | CarriageState::Loaded(s) => {
                Some(s.get(train_state).shapes().clone())
            },
            _ => None
        }
    }

    fn set_ready(&self) {
        let mut state = lock!(self.state);
        if let CarriageState::Pending(shapes) = &*state {
            *state = CarriageState::Loaded(shapes.clone());
        }
        self.try_lifecycle.set();
    }

    pub(super) fn has_shapes(&self) -> bool {
        match &*lock!(self.state) {
            CarriageState::Pending(_) | CarriageState::Loaded(_) => true,
            _ => false
        }
    }

    pub(super) fn ready(&self) -> bool {
        match &*lock!(self.state) {
            CarriageState::Loaded(_) => true,
            _ => false
        }
    }

    pub(super) async fn load(&mut self, base: &PeregrineCoreBase, result_store: &ShapeStore, mode: LoadMode) -> Result<(),DataMessage> {
        let unloaded = match &*lock!(self.state) {
            CarriageState::Unloaded(unloaded) => Some(unloaded.clone()),
            _ => None
        };
        if let Some(mut unloaded) = unloaded {
            *lock!(self.state) = CarriageState::Loading;
            if let Some(universe) = unloaded.load(&self.extent,base,result_store,mode).await? {
                *lock!(self.state) = CarriageState::Pending(CarriageShapes::new(&universe));
            }
        }
        Ok(())
    }

    pub(super) fn height_tracker(&self, train_state: &TrainState) -> HeightTracker {
        match &*lock!(self.state) {
            CarriageState::Pending(s) | CarriageState::Loaded(s) => {
                s.get(train_state).height_tracker()
            },
            _ => HeightTracker::empty()
        }        
    }
}

#[derive(Clone)]
pub struct DrawingCarriage {
    hash: Arc<u64>,
    carriage: Arc<Carriage>,
    train_state: Arc<TrainState>
}

impl PartialEq for DrawingCarriage {
    fn eq(&self, other: &Self) -> bool { self.hash == other.hash }
}

impl Eq for DrawingCarriage {}

impl Hash for DrawingCarriage {
    fn hash<H: Hasher>(&self, state: &mut H) { self.hash.hash(state); }
}

impl DrawingCarriage {
    fn calc_hash(carriage: &Carriage, train_state: &TrainState) -> u64 {
        let mut state = DefaultHasher::new();
        carriage.hash_by_serial(&mut state);
        train_state.hash(&mut state);
        state.finish()
    }

    pub fn new(carriage: &Carriage, train_state: &TrainState) -> DrawingCarriage {
        let hash = Self::calc_hash(carriage,train_state);
        DrawingCarriage {
            carriage: Arc::new(carriage.clone()),
            hash: Arc::new(hash),
            train_state: Arc::new(train_state.clone())
        }
    }

    pub fn extent(&self) -> &CarriageExtent { &self.carriage.extent() }
    pub fn shapes(&self) -> Option<Arc<Vec<Shape<LeafCommonStyle>>>> {
        self.carriage.shapes(&self.train_state)
    }
    pub fn set_ready(&self) { self.carriage.set_ready() }
}
