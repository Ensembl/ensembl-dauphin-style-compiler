use blackbox::blackbox_log;
use std::fmt::{ self, Display, Formatter };
use std::sync::{ Arc, Mutex };
use crate::api::{ PeregrineCore, MessageSender };
use crate::core::Track;
use crate::panel::{ Panel };
use crate::shape::{ Shape, ShapeList };
use super::train::TrainId;
use crate::util::message::DataMessage;
use web_sys::console;

#[derive(Clone,Hash,PartialEq,Eq)]
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
}

impl Display for CarriageId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f,"CarriageId(train={} index={})",self.train,self.index)
    }
}

#[derive(Clone)]
pub struct Carriage {
    id: CarriageId,
    shapes: Arc<Mutex<Option<ShapeList>>>,
    messages: MessageSender
}

impl fmt::Debug for Carriage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{:?}/{}",self.id.train,self.id.index)
    }
}

impl Carriage {
    pub fn new(id: &CarriageId, messages: &MessageSender) -> Carriage {
        Carriage {
            id: id.clone(),
            shapes: Arc::new(Mutex::new(None)),
            messages: messages.clone()
        }
    }

    pub fn id(&self) -> &CarriageId { &self.id }

    // XXX should be able to return without cloning
    pub fn shapes(&self) -> Vec<Shape> {
        let mut out = vec![];
        for shape in self.shapes.lock().unwrap().as_ref().map(|x| x.shapes()).unwrap_or(&vec![]) {
            out.push(shape.clone());
        }
        out
    }

    pub(super) fn ready(&self) -> bool {
        self.shapes.lock().unwrap().is_some()
    }

    fn make_panel(&self, track: &Track) -> Panel {
        Panel::new(self.id.train.layout().stick().as_ref().unwrap().clone(),self.id.index,self.id.train.scale().clone(),self.id.train.layout().focus().clone(),track.clone())
    }

    async fn load_full(&self, data: &PeregrineCore) -> anyhow::Result<()> {
        if self.ready() { return Ok(()); }
        let mut panels = vec![];
        for track in self.id.train.layout().tracks().iter() {
            panels.push((track,self.make_panel(track)));
        }
        // collect and reiterate to allow asyncs to run in parallel. Laziness in iters would defeat the point.
        let tracks : Vec<_> = panels.iter().map(|(t,p)| (t,data.panel_store.run(p))).collect();
        let mut new_shapes = ShapeList::new();
        for (track,future) in tracks {
            match future.await.as_ref() {
                Ok(zoo) => {
                    new_shapes.append(&zoo.track_shapes(track.name()));
                },
                Err(e) => {
                    self.messages.send(e.clone());
                }
            }
        }
        let mut shapes = self.shapes.lock().unwrap();
        if shapes.is_none() {
            *shapes = Some(new_shapes);
        }
        Ok(())
    }

    pub(super) async fn load(&self, data: &PeregrineCore) {
        match self.load_full(data).await {
            Ok(()) => (),
            Err(e) => {
                *self.shapes.lock().unwrap() = Some(ShapeList::new());
                data.integration.lock().unwrap().report_error(&e.to_string());
            }
        }
    }
}
