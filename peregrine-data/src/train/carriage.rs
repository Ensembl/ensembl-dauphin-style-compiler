use std::fmt::{ self, Display, Formatter };
use std::sync::{ Arc, Mutex };
use crate::api::{ PeregrineCore, MessageSender };
use crate::lane::{ ShapeRequest, Region };
use crate::shape::{ ShapeListBuilder, ShapeList };
use super::train::TrainId;
use crate::util::message::DataMessage;
use crate::switch::trackconfiglist::TrainTrackConfigList;

#[derive(Clone,Hash,PartialEq,Eq)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct CarriageId {
    train: TrainId,
    index: u64
}

impl CarriageId {
    pub fn new(train_id: &TrainId, index: u64) -> CarriageId {
        CarriageId {
            train: train_id.clone(),
            index
        }
    }

    pub fn left_right(&self) -> (f64,f64) {
        let bp_in_carriage = self.train.scale().bp_in_carriage() as f64;
        let index = self.index as f64;
        (bp_in_carriage*index,bp_in_carriage*(index+1.))
    }

    pub fn region(&self) -> Region {
        Region::new(self.train.layout().stick(),self.index,self.train.scale())
    }
}

/*
impl Display for CarriageId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f,"CarriageId(train={} index={})",self.train,self.index)
    }
}
*/

#[derive(Clone)]
pub struct Carriage {
    no_shapes: ShapeList, // useful to return ref to sometimes
    id: CarriageId,
    track_configs: TrainTrackConfigList,
    shapes: Arc<Mutex<Option<ShapeList>>>,
    messages: MessageSender
}

impl Carriage {
    pub fn new(id: &CarriageId, configs: &TrainTrackConfigList, messages: &MessageSender) -> Carriage {
        Carriage {
            no_shapes: ShapeList::empty(),
            id: id.clone(),
            shapes: Arc::new(Mutex::new(None)),
            track_configs: configs.clone(),
            messages: messages.clone()
        }
    }

    pub fn id(&self) -> &CarriageId { &self.id }

    // XXX should be able to return without cloning
    pub fn shapes(&self) -> ShapeList {
        let data = self.shapes.lock().unwrap();
        let shape = data.as_ref().cloned();
        shape.unwrap_or(ShapeList::empty())
    }

    pub(super) fn ready(&self) -> bool {
        self.shapes.lock().unwrap().is_some()
    }

    pub(super) async fn load(&mut self, data: &PeregrineCore) -> Result<(),DataMessage> {
        if self.ready() { return Ok(()); }
        let mut shape_requests = vec![];
        let track_config_list = self.id.train.layout().track_config_list();
        let track_list = self.track_configs.list_tracks();
        for track in track_list {
            if let Some(track_config) = track_config_list.get_track(&track) {
                shape_requests.push(ShapeRequest::new(&self.id.region(),&track_config));
            }
        }
        // collect and reiterate to allow asyncs to run in parallel. Laziness in iters would defeat the point.
        let mut errors = vec![];
        let lane_store = data.agent_store.lane_store.clone();
        let tracks : Vec<_> = shape_requests.iter().map(|p| lane_store.run(p)).collect();
        let mut new_shapes = ShapeListBuilder::new();
        for future in tracks {
            match future.await.as_ref() {
                Ok(zoo) => {
                    new_shapes.append(&zoo);
                },
                Err(e) => {
                    self.messages.send(e.clone());
                    errors.push(e.clone());
                }
            }
        }
        let shapes = new_shapes.build(&data.base.allotment_petitioner);
        self.shapes.lock().unwrap().replace(shapes);
        if errors.len() == 0 {
            Ok(())
        } else {
            Err(DataMessage::CarriageUnavailable(self.id.clone(),errors))
        }
    }
}
