use std::sync::{Mutex, Arc};

use peregrine_toolkit::{sync::needed::Needed, lock};

use crate::{CarriageExtent, switch::trackconfiglist::TrainTrackConfigList, api::MessageSender, ShapeRequestGroup, PeregrineCoreBase, ShapeStore, DataMessage, allotment::core::drawingcarriagedata::DrawingCarriageDataStore};

use super::loadshapes::{LoadMode, load_carriage_shape_list};

#[derive(Clone)]
pub(crate) struct CarriageProcess {
    try_lifecycle: Option<Needed>,
    extent: CarriageExtent,
    config: TrainTrackConfigList,
    messages: Option<MessageSender>,
    shapes: Arc<Mutex<Option<DrawingCarriageDataStore>>>,
    warm: bool
}

impl CarriageProcess {
    pub(crate) fn new(extent: &CarriageExtent, try_lifecycle: Option<&Needed>, configs: &TrainTrackConfigList, messages: Option<&MessageSender>, warm: bool) -> CarriageProcess {
        CarriageProcess {
            try_lifecycle: try_lifecycle.cloned(),
            extent: extent.clone(),
            config: configs.clone(),
            messages: messages.cloned(),
            shapes: Arc::new(Mutex::new(None)),
            warm
        }
    }

    pub fn extent(&self) -> &CarriageExtent { &self.extent }
    pub fn get_shapes(&self) -> Option<DrawingCarriageDataStore> { lock!(self.shapes).clone() }

    fn make_shape_requests(&self) -> ShapeRequestGroup {
        let track_config_list = self.extent.train().layout().track_config_list();
        let track_list = self.config.list_tracks();
        let pixel_size = self.extent.train().pixel_size();
        let mut track_configs = vec![];
        for track in track_list {
            if let Some(track_config) = track_config_list.get_track(&track) {
                track_configs.push(track_config.as_ref().clone());
            }
        }
        ShapeRequestGroup::new(&self.extent.region(),&track_configs,pixel_size,self.warm)
    }

    pub(crate) async fn load(&mut self, base: &PeregrineCoreBase, result_store: &ShapeStore, mode: LoadMode) -> Result<(),DataMessage> {
        let shape_requests = self.make_shape_requests();
        let shapes = 
            load_carriage_shape_list(base,result_store,self.messages.as_ref(),shape_requests,&mode).await
            .map_err(|errors| {
               DataMessage::CarriageUnavailable(self.extent.clone(),errors)
            })?;
        match mode {
            LoadMode::Network => { return Ok(()); },
            _ => {}
        }
        *lock!(self.shapes) = Some(DrawingCarriageDataStore::new(&shapes));
        if let Some(lifecycle) = &self.try_lifecycle {
            lifecycle.set();
        }
        Ok(())
    }
}
