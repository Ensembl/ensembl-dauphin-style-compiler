use std::sync::{ Arc, Mutex };
use crate::api::PeregrineObjects;
use crate::core::{ Layout, Scale };
use super::carriageset::CarriageSet;
use super::carriage::{ Carriage };
use std::fmt::{ self, Display, Formatter };

#[derive(Clone,PartialEq)]
pub struct TrainId {
    layout: Layout,
    scale: Scale
}

impl Display for TrainId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f,"TrainId(layout={} scale={})",self.layout,self.scale)
    }
}

impl TrainId {
    pub fn new(layout: &Layout, scale: &Scale) -> TrainId {
        TrainId {
            layout: layout.clone(),
            scale: scale.clone()
        }
    }

    pub fn layout(&self) -> &Layout { &self.layout }
    pub fn scale(&self) -> &Scale { &self.scale }
}

struct TrainData {
    ready: bool,
    id: TrainId,
    position: f64,
    carriages: Option<CarriageSet>
}

impl TrainData {
    fn new(data: &mut PeregrineObjects, id: &TrainId, position: f64) -> TrainData {
        let mut out = TrainData {
            ready: false,
            id: id.clone(),
            position,
            carriages: Some(CarriageSet::new(id,0))
        };
        out.set_position(data,position);
        out
    }

    fn id(&self) -> &TrainId { &self.id }
    fn position(&self) -> f64 { self.position }
    fn ready(&self) -> bool { self.ready }

    fn set_max(&mut self, max: u64) {
        if let Some(carriages) = &mut self.carriages {
            carriages.set_max(self.id.scale.carriage(max as f64));
        }
    }

    fn set_position(&mut self, data: &mut PeregrineObjects, position: f64) -> bool {
        self.position = position;
        let carriage = self.id.scale.carriage(position);
        let (carriages,changed) = CarriageSet::new_using(&self.id,carriage,self.carriages.take().unwrap());
        self.carriages = Some(carriages);
        changed
    }

    fn carriages(&self) -> Vec<Carriage> {
        self.carriages.as_ref().map(|x| x.carriages().to_vec()).unwrap_or_else(|| vec![])
    }

    fn set_ready(&mut self) {
        self.ready = true;
    }
}

// XXX circular
#[derive(Clone)]
pub struct Train(Arc<Mutex<TrainData>>);

impl Train {
    pub fn new(data: &mut PeregrineObjects, id: &TrainId, position: f64) -> Train {
        Train(Arc::new(Mutex::new(TrainData::new(data,id,position))))
    }

    pub fn id(&self) -> TrainId { self.0.lock().unwrap().id().clone() }
    pub fn position(&self) -> f64 { self.0.lock().unwrap().position() }
    pub fn ready(&self) -> bool { self.0.lock().unwrap().ready() }
    pub fn carriages(&self) -> Vec<Carriage> { self.0.lock().unwrap().carriages() }

    pub fn set_position(&self, data: &mut PeregrineObjects, position: f64) -> bool {
        self.0.lock().unwrap().set_position(data,position)
    }

    async fn find_max(&self, data: &mut PeregrineObjects) {
        let train_id = self.id();
        if let Ok(stick) = data.stick_store.get(train_id.layout().stick()).await {
            if let Some(stick) = &stick.as_ref() {
                self.0.lock().unwrap().set_max(stick.size());
            }
        }
    }

    pub async fn load(&self, data: &mut PeregrineObjects) {
        self.find_max(data).await;
        let carriages = self.0.lock().unwrap().carriages();
        for carriage in carriages {
            carriage.load(data).await;
        }
        self.0.lock().unwrap().set_ready();
    }
}
