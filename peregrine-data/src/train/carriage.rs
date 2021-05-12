use std::fmt::{ self, Display, Formatter };
use std::sync::{ Arc, Mutex };
use crate::api::{ PeregrineCore, MessageSender };
use crate::lane::{ ShapeRequest, Region };
use crate::shape::{ Shape, ShapeList };
use super::train::TrainId;
use crate::util::message::DataMessage;
use crate::switch::trackconfig::{ TrackConfig };
use crate::switch::trackconfiglist::TrainTrackConfigList;
use crate::switch::allotment::Allotter;

#[derive(Clone,Debug,Hash,PartialEq,Eq)]
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

    pub fn left(&self) -> f64 {
        (self.train.scale().bp_in_carriage() * self.index) as f64
    }

    pub fn region(&self) -> Region {
        Region::new(self.train.layout().stick(),self.index,self.train.scale())
    }
}

impl Display for CarriageId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f,"CarriageId(train={} index={})",self.train,self.index)
    }
}

#[derive(Clone)]
pub struct Carriage {
    id: CarriageId,
    track_configs: TrainTrackConfigList,
    shapes: Arc<Mutex<Option<(ShapeList,Allotter)>>>,
    messages: MessageSender
}

impl Carriage {
    pub fn new(id: &CarriageId, configs: &TrainTrackConfigList, messages: &MessageSender) -> Carriage {
        Carriage {
            id: id.clone(),
            shapes: Arc::new(Mutex::new(None)),
            track_configs: configs.clone(),
            messages: messages.clone()
        }
    }

    pub fn id(&self) -> &CarriageId { &self.id }

    // XXX should be able to return without cloning
    pub fn shapes(&self) -> Vec<Shape> {
        let mut out = vec![];
        for shape in self.shapes.lock().unwrap().as_ref().map(|(shapes,_)| shapes.shapes()).unwrap_or(&vec![]) {
            out.push(shape.clone());
        }
        out
    }

    pub(super) fn ready(&self) -> bool {
        self.shapes.lock().unwrap().is_some()
    }

    pub(super) async fn load(&self, data: &PeregrineCore) -> Result<(),DataMessage> {
        if self.ready() { return Ok(()); }
        let mut shape_requests = vec![];
        let track_config_list = self.id.train.layout().track_config_list();
        let track_list = self.track_configs.list_tracks();
        for track in track_list {
            use web_sys::console;
            console::log_1(&format!("track: {} ({:?})",track.program_name().1,self.id).into());
            if let Some(track_config) = track_config_list.get_track(&track) {
                shape_requests.push(ShapeRequest::new(&self.id.region(),&track_config));
            }
        }
        // collect and reiterate to allow asyncs to run in parallel. Laziness in iters would defeat the point.
        let mut errors = vec![];
        let lane_store = data.agent_store.lane_store().await;
        let tracks : Vec<_> = shape_requests.iter().map(|p| lane_store.run(p)).collect();
        let mut new_shapes = ShapeList::new();
        for future in tracks {
            match future.await.as_ref() {
                Ok(zoo) => {
                    use web_sys::console;
                    console::log_1(&format!("got new shapes").into());
                    new_shapes.append(&zoo);
                },
                Err(e) => {
                    self.messages.send(e.clone());
                    errors.push(e.clone());
                }
            }
        }
        let allotments = new_shapes.make_allotter(&data.base.allotment_petitioner);
        let mut shapes = self.shapes.lock().unwrap();
        if shapes.is_none() {
            *shapes = Some((new_shapes,allotments));
        }
        if errors.len() == 0 {
            Ok(())
        } else {
            Err(DataMessage::CarriageUnavailable(self.id.clone(),errors))
        }
    }
}
