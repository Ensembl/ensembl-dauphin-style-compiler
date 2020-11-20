use std::fmt::{ self, Display, Formatter };
use std::sync::{ Arc, Mutex };
use crate::api::PeregrineObjects;
use crate::core::Track;
use crate::panel::{ Panel };
use crate::shape::ShapeList;
use super::train::TrainId;

#[derive(Clone)]
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
}

impl Display for CarriageId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f,"CarriageId(train={} index={})",self.train,self.index)
    }
}

#[derive(Clone)]
pub struct Carriage {
    id: CarriageId,
    shapes: Arc<Mutex<Option<ShapeList>>>
}

impl Carriage {
    pub fn new(id: &CarriageId) -> Carriage {
        Carriage {
            id: id.clone(),
            shapes: Arc::new(Mutex::new(None))
        }
    }

    pub fn id(&self) -> &CarriageId { &self.id }

    pub fn ready(&self) -> bool {
        self.shapes.lock().unwrap().is_some()
    }

    fn make_panel(&self, track: &Track) -> Panel {
        Panel::new(self.id.train.layout().stick().as_ref().unwrap().clone(),self.id.index,self.id.train.scale().clone(),self.id.train.layout().focus().clone(),track.clone())
    }

    async fn load_full(&self, data: &PeregrineObjects) -> anyhow::Result<()> {
        let mut shapes = self.shapes.lock().unwrap();
        if shapes.is_some() { return Ok(()); }
        let mut panels = vec![];
        for track in self.id.train.layout().tracks().iter() {
            panels.push((track,self.make_panel(track)));
        }
        // collect and reiterate to allow asyncs to run in parallel. Laziness in iters would defeat the point.
        let tracks : Vec<_> = panels.iter().map(|(t,p)| (t,data.panel_store.run(p))).collect();
        let mut new_shapes = ShapeList::new();
        for (track,future) in tracks {
            let zoo = future.await?;
            new_shapes.append(&zoo.track_shapes(track.name()));
        }
        *shapes = Some(new_shapes);
        Ok(())
    }

    pub async fn load(&self, data: &PeregrineObjects) {
        match self.load_full(data).await {
            Ok(()) => (),
            Err(e) => {
                *self.shapes.lock().unwrap() = Some(ShapeList::new());
                data.integration.lock().unwrap().report_error(&e.to_string());
            }
        }
    }
}
