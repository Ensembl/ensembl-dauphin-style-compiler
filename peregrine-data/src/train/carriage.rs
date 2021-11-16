use std::sync::{ Arc, Mutex };
use peregrine_toolkit::lock;
use crate::{CarriageExtent, LaneStore, PeregrineCoreBase, PgCommanderTaskSpec, add_task};
use crate::api::{ MessageSender };
use crate::lane::{ ShapeRequest };
use crate::shape::{ ShapeListBuilder, ShapeList };
use crate::util::message::DataMessage;
use crate::switch::trackconfiglist::TrainTrackConfigList;

#[derive(Clone)]
pub enum CarriageLoadMode {
    RealTime,
    Batch,
    Network
}

impl CarriageLoadMode {
    pub fn build_shapes(&self) -> bool {
        match self {
            CarriageLoadMode::Network => false,
            _ => true
        }
    }

    pub fn high_priority(&self) -> bool {
        match self {
            CarriageLoadMode::RealTime => true,
            _ => false
        }
    }
}

#[derive(Clone)]
pub struct Carriage {
    no_shapes: ShapeList, // useful to return ref to sometimes
    extent: CarriageExtent,
    track_configs: TrainTrackConfigList,
    shapes: Arc<Mutex<Option<ShapeList>>>,
    messages: Option<MessageSender>
}

impl Carriage {
    pub fn new(extent: &CarriageExtent, configs: &TrainTrackConfigList, messages: Option<&MessageSender>) -> Carriage {
        Carriage {
            no_shapes: ShapeList::empty(),
            extent: extent.clone(),
            shapes: Arc::new(Mutex::new(None)),
            track_configs: configs.clone(),
            messages: messages.cloned()
        }
    }

    pub fn extent(&self) -> &CarriageExtent { &self.extent }

    // XXX should be able to return without cloning
    pub fn shapes(&self) -> ShapeList {
        let data = self.shapes.lock().unwrap();
        let shape = data.as_ref().cloned();
        shape.unwrap_or(ShapeList::empty())
    }

    pub(super) fn ready(&self) -> bool {
        self.shapes.lock().unwrap().is_some()
    }

    pub(super) async fn load(&mut self, base: &PeregrineCoreBase, result_store: &LaneStore, mode: CarriageLoadMode) -> Result<(),DataMessage> {
        if self.ready() { return Ok(()); }
        let mut shape_requests = vec![];
        let track_config_list = self.extent.train().layout().track_config_list();
        let track_list = self.track_configs.list_tracks();
        for track in track_list {
            if let Some(track_config) = track_config_list.get_track(&track) {
                shape_requests.push((ShapeRequest::new(&self.extent.region(),&track_config),mode.clone()));
            }
        }
        // collect and reiterate to allow asyncs to run in parallel. Laziness in iters would defeat the point.
        let mut errors = vec![];
        let lane_store = result_store.clone();
        let tracks : Vec<_> = shape_requests.iter().map(|p|{
            let (request,mode) = p.clone();
            let lane_store = lane_store.clone();
            add_task(&base.commander,PgCommanderTaskSpec {
                name: format!("data program"),
                prio: if mode.high_priority() { 0 } else { 9 },
                slot: None,
                timeout: None,
                stats: false,
                task: Box::pin(async move {
                    lane_store.run(&request,&mode).await.as_ref().clone()
                })
            })
        }).collect();
        if !mode.build_shapes() { return Ok(()); }
        let mut new_shapes = ShapeListBuilder::new(&base.allotment_metadata,&*lock!(base.assets));
        for future in tracks {
            future.finish_future().await;
            match future.take_result().as_ref().unwrap() {
                Ok(zoo) => {
                    new_shapes.append(&zoo);
                },
                Err(e) => {
                    if let Some(messages) = &self.messages {
                        messages.send(e.clone());
                    }
                    errors.push(e.clone());
                }
            }
        }
        let shapes = new_shapes.build();
        self.shapes.lock().unwrap().replace(shapes);
        if errors.len() == 0 {
            Ok(())
        } else {
            Err(DataMessage::CarriageUnavailable(self.extent.clone(),errors))
        }
    }
}
