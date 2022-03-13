use std::sync::{ Arc, Mutex };
use peregrine_toolkit::lock;
use peregrine_toolkit::puzzle::PuzzleSolution;
use peregrine_toolkit::sync::needed::Needed;

use crate::allotment::core::allotmentmetadata2::AllotmentMetadataReport2;
use crate::api::MessageSender;
use crate::{CarriageExtent, ShapeStore, PeregrineCoreBase, AnchoredCarriageShapeList, CarriageShapeList2, Shape};
use crate::shapeload::{ ShapeRequest, ShapeRequestGroup };
use crate::util::message::DataMessage;
use crate::switch::trackconfiglist::TrainTrackConfigList;
use crate::shapeload::loadshapes::{LoadMode, load_carriage_shape_list };

use super::railwayevent::RailwayEvents;

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

    async fn load(&mut self, extent: &CarriageExtent, base: &PeregrineCoreBase, result_store: &ShapeStore, mode: LoadMode) -> Result<Option<CarriageShapeList2>,DataMessage> {
        let shape_requests = self.make_shape_requests(extent);
        let (shapes,errors) = load_carriage_shape_list(base,result_store,self.messages.as_ref(),shape_requests,&mode).await;
        let shapes = if let Some(x) = shapes { x } else { return Ok(None); };
        if errors.len() != 0 {
            return Err(DataMessage::CarriageUnavailable(extent.clone(),errors));
        }    
        Ok(Some(shapes))
    }
}

enum CarriageState {
    Unloaded(UnloadedCarriage),
    Loading,
    Pending(CarriageShapeList2),
    Loaded(CarriageShapeList2)
}

#[derive(Clone,Copy,Debug,PartialEq,Eq,Hash)]
pub struct CarriageSerial(u64);

#[derive(Clone)]
pub(crate) struct CarriageSerialSource(Arc<Mutex<u64>>);

impl CarriageSerialSource {
    pub(crate) fn new() -> CarriageSerialSource { CarriageSerialSource(Arc::new(Mutex::new(0))) }
    fn next(&self) -> CarriageSerial {
        let mut v = lock!(self.0);
        *v += 1;
        CarriageSerial(*v)
    }
}

#[derive(Clone)]
pub struct Carriage {
    try_lifecycle: Needed,
    moribund: Arc<Mutex<bool>>,
    serial: CarriageSerial,
    extent: CarriageExtent,
    state: Arc<Mutex<CarriageState>>
}

impl Carriage {
    pub(crate) fn new(try_lifecycle: &Needed, serial_source: &CarriageSerialSource, extent: &CarriageExtent, configs: &TrainTrackConfigList, messages: Option<&MessageSender>, warm: bool) -> Carriage {
        Carriage {
            try_lifecycle: try_lifecycle.clone(),
            moribund: Arc::new(Mutex::new(false)),
            serial: serial_source.next(),
            extent: extent.clone(),
            state: Arc::new(Mutex::new(CarriageState::Unloaded(UnloadedCarriage {
                config: configs.clone(),
                messages: messages.cloned(),
                warm
            })))
        }
    }

    pub fn is_moribund(&self) -> bool { *lock!(self.moribund) }
    pub fn serial(&self) -> CarriageSerial { self.serial }
    pub fn extent(&self) -> &CarriageExtent { &self.extent }

    pub(super) fn set_moribund(&self,carriage_events: &mut RailwayEvents) {
        *lock!(self.moribund) = true;
        match &*lock!(self.state) {
            CarriageState::Loaded(_) => {
                carriage_events.draw_drop_carriage(self);
            },
            _ => {}
        }
    }

    pub fn metadata(&self) -> Result<AllotmentMetadataReport2,DataMessage> {
        match &*lock!(self.state) {
            CarriageState::Pending(s) | CarriageState::Loaded(s) => {
                let mut solution = PuzzleSolution::new(s.puzzle());
                // TODO the inter-carriage stuff
                solution.solve();
                Ok(s.get_metadata(&solution))
            },
            _ => Ok(AllotmentMetadataReport2::empty())
        }

    }

    pub fn shapes(&self) -> Result<Vec<Shape<()>>,DataMessage> {
        match &*lock!(self.state) {
            CarriageState::Pending(s) | CarriageState::Loaded(s) => {
                let mut solution = PuzzleSolution::new(s.puzzle());
                // TODO the inter-carriage stuff
                solution.solve();
                Ok(s.get(&solution)) 
            },
            _ => Ok(vec![])
        }
    }

    pub fn set_ready(&self) {
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
            if let Some(new_state) = unloaded.load(&self.extent,base,result_store,mode).await? {
                *lock!(self.state) = CarriageState::Pending(new_state);
            }
        }
        Ok(())
    }
}
