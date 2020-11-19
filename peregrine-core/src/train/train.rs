use std::sync::{ Arc, Mutex };
use crate::api::{ PeregrineObjects, CarriageSpeed };
use crate::core::{ Layout, Scale };
use super::carriageset::CarriageSet;
use super::carriageevent::CarriageEvents;
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
    active: Option<u32>,
    id: TrainId,
    position: f64,
    carriages: Option<CarriageSet>
}

impl TrainData {
    fn new(id: &TrainId, carriage_event: &mut CarriageEvents, position: f64) -> TrainData {
        let mut out = TrainData {
            ready: false,
            active: None,
            id: id.clone(),
            position,
            carriages: Some(CarriageSet::new(id,carriage_event,0))
        };
        out.set_position(carriage_event,position);
        out
    }

    fn set_active(&mut self, carriage_event: &mut CarriageEvents, index: u32, quick: bool) {
        if self.active != Some(index) {
            let speed = if quick { CarriageSpeed::Quick } else { CarriageSpeed::Slow };
            carriage_event.transition(index,speed);
        }
        self.active = Some(index);
    }

    fn set_inactive(&mut self) {
        self.active = None;
    }

    fn id(&self) -> &TrainId { &self.id }
    fn position(&self) -> f64 { self.position }
    fn ready(&self) -> bool { self.ready }

    fn set_max(&mut self, max: u64) {
        if let Some(carriages) = &mut self.carriages {
            carriages.set_max(self.id.scale.carriage(max as f64));
        }
    }

    fn set_position(&mut self, carriage_event: &mut CarriageEvents, position: f64) {
        self.position = position;
        let carriage = self.id.scale.carriage(position);
        let (carriages,changed) = CarriageSet::new_using(&self.id,carriage_event,carriage,self.carriages.take().unwrap());
        if let Some(index) = self.active {
            if changed {
                carriages.send_event(carriage_event,index);
            }
        }
        self.carriages = Some(carriages);
    }

    fn carriages(&self) -> Vec<Carriage> {
        self.carriages.as_ref().map(|x| x.carriages().to_vec()).unwrap_or_else(|| vec![])
    }

    fn set_ready(&mut self) {
        self.ready = true;
    }

    pub fn maybe_ready(&mut self) {
        if let Some(carriages) = &self.carriages {
            if carriages.ready() {
                self.ready = true;
            }
        }
    }
}

// XXX circular chroms
#[derive(Clone)]
pub struct Train(Arc<Mutex<TrainData>>);

impl Train {
    pub fn new(id: &TrainId, carriage_event: &mut CarriageEvents, position: f64) -> Train {
        Train(Arc::new(Mutex::new(TrainData::new(id,carriage_event,position))))
    }

    pub fn id(&self) -> TrainId { self.0.lock().unwrap().id().clone() }
//    pub fn position(&self) -> f64 { self.0.lock().unwrap().position() }
    pub fn ready(&self) -> bool { self.0.lock().unwrap().ready() }
//    pub fn carriages(&self) -> Vec<Carriage> { self.0.lock().unwrap().carriages() }

    pub fn set_active(&mut self, carriage_event: &mut CarriageEvents, index: u32, quick: bool) {
        self.0.lock().unwrap().set_active(carriage_event,index,quick);
    }

    pub fn set_inactive(&mut self) {
        self.0.lock().unwrap().set_inactive();
    }

    pub fn set_position(&self, carriage_event: &mut CarriageEvents, position: f64) {
        self.0.lock().unwrap().set_position(carriage_event,position);
    }

    pub fn compatible_with(&self, other: &Train) -> bool {
        self.id().layout().stick() == other.id().layout().stick()
    }

    pub fn maybe_ready(&mut self) {
        self.0.lock().unwrap().maybe_ready();
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
