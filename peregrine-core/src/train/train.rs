use std::sync::{ Arc, Mutex };
use super::trackstate::TrackStateSnapshot;
use crate::core::{ Focus, Scale, StickId, PeregrineData };
use super::carriageset::CarriageSet;
use super::carriage::{ Carriage };
use std::fmt::{ self, Display, Formatter };

#[derive(Clone,PartialEq)]
pub struct RailwayId {
    tracks: TrackStateSnapshot,
    focus: Focus,
    stick: StickId
}

impl RailwayId {
    pub fn new(tracks: TrackStateSnapshot, stick_id: &StickId, scale: &Scale, focus: &Focus) -> RailwayId {
        RailwayId {
            tracks,
            focus: focus.clone(),
            stick: stick_id.clone()
        }
    }

    pub fn tracks(&self) -> &TrackStateSnapshot { &self.tracks }
    pub fn focus(&self) -> &Focus { &self.focus }
    pub fn stick(&self) -> &StickId { &self.stick }
}

impl Display for RailwayId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f,"RailwayId(tracks={} focus={} stick={})",self.tracks,self.focus,self.stick)
    }
}

#[derive(Clone,PartialEq)]
pub struct TrainId {
    railway: RailwayId,
    scale: Scale
}

impl Display for TrainId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f,"TrainId(railway={} scale={})",self.railway,self.scale)
    }
}

impl TrainId {
    pub fn new(railway: &RailwayId, scale: &Scale) -> TrainId {
        TrainId {
            railway: railway.clone(),
            scale: scale.clone()
        }
    }

    pub fn railway(&self) -> &RailwayId { &self.railway }
    pub fn scale(&self) -> &Scale { &self.scale }
}

struct TrainData {
    ready: bool,
    id: TrainId,
    position: f64,
    carriages: Option<CarriageSet>
}

impl TrainData {
    fn new(data: &mut PeregrineData, id: &TrainId, position: f64) -> TrainData {
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

    fn set_position(&mut self, data: &mut PeregrineData, position: f64) -> bool {
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
    pub fn new(data: &mut PeregrineData, id: &TrainId, position: f64) -> Train {
        Train(Arc::new(Mutex::new(TrainData::new(data,id,position))))
    }

    pub fn id(&self) -> TrainId { self.0.lock().unwrap().id().clone() }
    pub fn position(&self) -> f64 { self.0.lock().unwrap().position() }
    pub fn ready(&self) -> bool { self.0.lock().unwrap().ready() }
    pub fn carriages(&self) -> Vec<Carriage> { self.0.lock().unwrap().carriages() }

    pub fn set_position(&self, data: &mut PeregrineData, position: f64) -> bool {
        self.0.lock().unwrap().set_position(data,position)
    }

    async fn find_max(&self, data: &mut PeregrineData) {
        let train_id = self.id();
        if let Ok(stick) = data.stick_store.get(train_id.railway().stick()).await {
            if let Some(stick) = &stick.as_ref() {
                self.0.lock().unwrap().set_max(stick.size());
            }
        }
    }

    pub async fn load(&self, data: &mut PeregrineData) {
        self.find_max(data).await;
        let carriages = self.0.lock().unwrap().carriages();
        for carriage in carriages {
            carriage.load(data).await;
        }
        self.0.lock().unwrap().set_ready();
    }
}
