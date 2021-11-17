use std::sync::{ Arc, Mutex };
use peregrine_toolkit::lock;

use crate::api::MessageSender;
use crate::{CarriageExtent, LaneStore, PeregrineCoreBase };
use crate::lane::{ ShapeRequest };
use crate::shape::{ ShapeList };
use crate::util::message::DataMessage;
use crate::switch::trackconfiglist::TrainTrackConfigList;
use crate::lane::shapeloader::{LoadMode, load_shapes};

#[derive(Clone)]
struct UnloadedCarriage {
    config: TrainTrackConfigList,
    messages: Option<MessageSender>
}

impl UnloadedCarriage {
    fn make_shape_requests(&self, extent: &CarriageExtent) -> Vec<ShapeRequest> {
        let mut shape_requests = vec![];
        let track_config_list = extent.train().layout().track_config_list();
        let track_list = self.config.list_tracks();
        for track in track_list {
            if let Some(track_config) = track_config_list.get_track(&track) {
                shape_requests.push(ShapeRequest::new(&extent.region(),&track_config));
            }
        }
        shape_requests
    }

    async fn load(&mut self, extent: &CarriageExtent, base: &PeregrineCoreBase, result_store: &LaneStore, mode: LoadMode) -> Result<Option<LoadedCarriage>,DataMessage> {
        let shape_requests = self.make_shape_requests(extent);
        let (shapes,errors) = load_shapes(base,result_store,self.messages.as_ref(),shape_requests,&mode).await;
        Ok(match shapes {
            Some(shapes) => {
                if errors.len() != 0 {
                    return Err(DataMessage::CarriageUnavailable(extent.clone(),errors));
                }    
                Some(LoadedCarriage {shapes})
            },
            None => None
        })
    }
}

struct LoadedCarriage {
    shapes: ShapeList
}

enum CarriageState {
    Unloaded(UnloadedCarriage),
    Loading,
    Loaded(LoadedCarriage)
}

#[derive(Clone)]
pub struct Carriage {
    extent: CarriageExtent,
    state: Arc<Mutex<CarriageState>>
}

impl Carriage {
    pub fn new(extent: &CarriageExtent, configs: &TrainTrackConfigList, messages: Option<&MessageSender>) -> Carriage {
        Carriage {
            extent: extent.clone(),
            state: Arc::new(Mutex::new(CarriageState::Unloaded(UnloadedCarriage {
                config: configs.clone(),
                messages: messages.cloned()
            })))
        }
    }

    pub fn extent(&self) -> &CarriageExtent { &self.extent }

    pub fn shapes(&self) -> ShapeList {
        match &*lock!(self.state) {
            CarriageState::Loaded(s) => { s.shapes.clone() },
            _ => ShapeList::empty()
        }
    }

    pub(super) fn ready(&self) -> bool {
        match &*lock!(self.state) {
            CarriageState::Loaded(_) => true,
            _ => false
        }
    }

    pub(super) async fn load(&mut self, base: &PeregrineCoreBase, result_store: &LaneStore, mode: LoadMode) -> Result<(),DataMessage> {
        let unloaded = match &*lock!(self.state) {
            CarriageState::Unloaded(unloaded) => Some(unloaded.clone()),
            _ => None
        };
        if let Some(mut unloaded) = unloaded {
            *lock!(self.state) = CarriageState::Loading;
            if let Some(new_state) = unloaded.load(&self.extent,base,result_store,mode).await? {
                *lock!(self.state) = CarriageState::Loaded(new_state);
            }
        }
        Ok(())
    }
}
