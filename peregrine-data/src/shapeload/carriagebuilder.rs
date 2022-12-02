use std::sync::{Mutex, Arc};
use peregrine_toolkit::{lock, error::Error};
use crate::{switch::trackconfiglist::TrainTrackConfigList, api::MessageSender, ShapeRequestGroup, PeregrineCoreBase, ShapeStore, allotment::core::{floatingcarriage::FloatingCarriage}, PeregrineApiQueue};
use crate::train::model::carriageextent::CarriageExtent;
use super::loadshapes::{LoadMode, load_carriage_shape_list};

#[derive(Clone)]
pub(crate) struct CarriageBuilder {
    api_queue: PeregrineApiQueue,
    extent: CarriageExtent,
    config: TrainTrackConfigList,
    messages: Option<MessageSender>,
    output: Arc<Mutex<Option<FloatingCarriage>>>,
    warm: bool
}

impl CarriageBuilder {
    pub(crate) fn new(api_queue: &PeregrineApiQueue, extent: &CarriageExtent, configs: &TrainTrackConfigList, messages: Option<&MessageSender>, warm: bool) -> CarriageBuilder {
        CarriageBuilder {
            api_queue: api_queue.clone(),
            extent: extent.clone(),
            config: configs.clone(),
            messages: messages.cloned(),
            output: Arc::new(Mutex::new(None)),
            warm
        }
    }

    #[cfg(debug_trains)]
    pub fn extent(&self) -> &CarriageExtent { &self.extent }
    
    pub fn get_carriage_output(&self) -> Option<FloatingCarriage> {
        lock!(self.output).as_ref().map(|x| x.clone())
    }

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

    pub(crate) async fn load(&mut self, base: &PeregrineCoreBase, result_store: &ShapeStore, mode: LoadMode) -> Result<(),Error> {
        let shape_requests = self.make_shape_requests();
        let shapes = 
            load_carriage_shape_list(base,result_store,self.messages.as_ref(),shape_requests,Some(&self.extent),&mode).await
            .map_err(|errors| {
                let errors = errors.iter().map(|x| x.message.clone()).collect::<Vec<_>>();
                Error::operr(&format!("carriage unavailable: {}",errors.join(", ")))
            })?;
        match mode {
            LoadMode::Network => { return Ok(()); },
            _ => {}
        }
        *lock!(self.output) = Some(shapes);
        self.api_queue.ping();
        Ok(())
    }
}
