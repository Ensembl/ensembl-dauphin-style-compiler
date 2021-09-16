use std::sync::{ Arc, Mutex };
use std::fmt;
use crate::api::{ PeregrineCore, CarriageSpeed, MessageSender };
use crate::core::{ Layout, Scale };
use crate::switch::pitch::Pitch;
use super::carriage::Carriage;
use super::carriageset::CarriageSet;
use super::carriageevent::CarriageEvents;
use crate::run::{ add_task, async_complete_task };
use crate::util::message::DataMessage;
use crate::{AllotmentRequestBuilder, AllotterMetadata, PgCommanderTaskSpec};
use crate::switch::trackconfiglist::TrainTrackConfigList;
use crate::core::Viewport;

#[derive(Clone,Hash,PartialEq,Eq)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct TrainId {
    layout: Layout,
    scale: Scale
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
    broken: bool,
    data_ready: bool,
    active: Option<u32>,
    id: TrainId,
    viewport: Viewport,
    max: Option<u64>,
    carriages: Option<CarriageSet>,
    messages: MessageSender,
    track_configs: TrainTrackConfigList,
    allotment_metadata: Arc<Vec<Arc<AllotmentRequestBuilder>>>
}

impl TrainData {
    fn new(id: &TrainId, carriage_event: &mut CarriageEvents, viewport: &Viewport, messages: &MessageSender) -> Result<TrainData,DataMessage> {
        let train_track_config_list = TrainTrackConfigList::new(&id.layout,&id.scale);
        let mut out = TrainData {
            broken: false,
            data_ready: false,
            active: None,
            id: id.clone(),
            viewport: viewport.clone(),
            carriages: Some(CarriageSet::new()),
            max: None,
            messages: messages.clone(),
            track_configs: train_track_config_list,
            allotment_metadata: Arc::new(vec![])
        };
        out.set_position(carriage_event,viewport)?;
        Ok(out)
    }

    fn set_active(&mut self, carriage_event: &mut CarriageEvents, index: u32, speed: CarriageSpeed) {
        if self.active != Some(index) {
            self.active = Some(index);
            self.set_carriages(carriage_event);
            carriage_event.transition(index,self.max.unwrap(),speed);
        }
    }

    fn set_inactive(&mut self) { self.active = None; }
    fn is_active(&self) -> bool { self.active.is_some() }
    fn viewport(&self) -> &Viewport { &self.viewport }
    fn id(&self) -> &TrainId { &self.id }
    fn train_ready(&self) -> bool { self.data_ready && self.max.is_some() }
    fn is_broken(&self) -> bool { self.broken }

    fn central_carriage(&self) -> Option<&Carriage> {
        if let Some(carriages) = &self.carriages {
            let carriages = carriages.carriages();
            if carriages.len() > 0 {
                let central = &carriages[carriages.len()/2];
                if central.ready() {
                    return Some(central);
                }
            }
        }
        None
    }

    fn allotter_metadata(&self) -> Option<AllotterMetadata> {
        self.central_carriage().map(|c| c.shapes().allotter().metadata().clone())
    }

    fn pitch(&self) -> Pitch {
        let mut pitch = Pitch::new();
        if let Some(carriages) = &self.carriages {
            for carriage in carriages.carriages() {
                if carriage.ready() {
                    let allotter = carriage.shapes().allotter();
                    pitch.merge(allotter.pitch());
                }
            }
        }
        pitch
    }

    fn set_max(&mut self, max: Result<u64,DataMessage>) {
        match max {
            Ok(max) => { self.max = Some(max); },
            Err(e) => {
                self.messages.send(e.clone());
                self.broken = true;
            }
        }
    }

    // TODO don't always update CarriageSet
    fn set_position(&mut self, carriage_event: &mut CarriageEvents, viewport: &Viewport) -> Result<(),DataMessage> {
        self.viewport = viewport.clone();
        let carriage = self.id.scale.carriage(viewport.position()?);
        let carriages = CarriageSet::new_using(&self.id,&self.track_configs,carriage_event,carriage,self.carriages.take().unwrap(),&self.messages);
        self.carriages = Some(carriages);
        Ok(())
    }

    fn maybe_ready(&mut self) {
        if let Some(carriages) = &self.carriages {
            if carriages.ready() && self.max.is_some() {
                self.data_ready = true;
            }
        }
    }

    fn carriages(&self) -> Vec<Carriage> {
        self.carriages.as_ref().map(|x| x.carriages().to_vec()).unwrap_or_else(|| vec![])
    }

    fn set_carriages(&mut self, events: &mut CarriageEvents) {
        if let Some(carriages) = &mut self.carriages {
            if carriages.ready() {
                if let Some(index) = self.active {
                    events.set_carriages(&self.carriages(),self.id.scale().clone(),index);
                }
            }
        }
    }
}

// XXX circular chroms
#[derive(Clone)]
pub struct Train(Arc<Mutex<TrainData>>,MessageSender);

impl Train {
    pub(super) fn new(id: &TrainId, carriage_event: &mut CarriageEvents, viewport: &Viewport, messages: &MessageSender) -> Result<Train,DataMessage> {
        let out = Train(Arc::new(Mutex::new(TrainData::new(id,carriage_event,viewport,&messages)?)),messages.clone());
        carriage_event.train(&out);
        Ok(out)
    }

    pub fn id(&self) -> TrainId { self.0.lock().unwrap().id().clone() }
    pub fn viewport(&self) -> Viewport { self.0.lock().unwrap().viewport().clone() }
    pub fn is_active(&self) -> bool { self.0.lock().unwrap().is_active() }
    pub(super) fn train_ready(&self) -> bool { self.0.lock().unwrap().train_ready() }
    pub(super) fn train_broken(&self) -> bool { self.0.lock().unwrap().is_broken() }
    pub(super) fn allotter_metadata(&self) -> Option<AllotterMetadata> { self.0.lock().unwrap().allotter_metadata() }

    pub(super) fn set_active(&mut self, carriage_event: &mut CarriageEvents, index: u32, speed: CarriageSpeed) {
        self.0.lock().unwrap().set_active(carriage_event,index,speed);
    }

    pub(super) fn set_inactive(&mut self) {
        self.0.lock().unwrap().set_inactive();
    }

    pub(super) fn set_position(&self, carriage_event: &mut CarriageEvents, viewport: &Viewport) -> Result<(),DataMessage> {
        self.0.lock().unwrap().set_position(carriage_event,viewport)?;
        Ok(())
    }

    pub(super) fn speed_limit(&self, other: &Train) -> CarriageSpeed {
        let same_stick = self.id().layout().stick() == other.id().layout().stick();
        if same_stick {
            let same_layout = self.id().layout() == other.id().layout();
            if same_layout {
                CarriageSpeed::Quick
            } else {
                CarriageSpeed::SlowCrossFade
            }
        } else {
            CarriageSpeed::Slow
        }
    }

    pub(super) fn maybe_ready(&mut self) { self.0.lock().unwrap().maybe_ready(); }
    pub(super) fn pitch(&self) -> Pitch { self.0.lock().unwrap().pitch() }

    async fn find_max(&self, data: &mut PeregrineCore) -> Result<u64,DataMessage> {
        Ok(data.agent_store.stick_store.get(&self.id().layout().stick()).await?.size())
    }

    fn set_max(&self, max: Result<u64,DataMessage>) {
        self.0.lock().unwrap().set_max(max);
    }
    
    pub(super) fn run_find_max(&self, objects: &mut PeregrineCore) {
        let self2 = self.clone();
        let mut objects2 = objects.clone();
        let handle = add_task(&objects.base.commander,PgCommanderTaskSpec {
            name: format!("max finder"),
            prio: 1,
            slot: None,
            timeout: None,
            task: Box::pin(async move {
                let max = self2.find_max(&mut objects2).await;
                if let Err(e) = &max { 
                    objects2.base.messages.send(e.clone());
                }
                self2.set_max(max);
                objects2.train_set.clone().update_trains(&mut objects2);
                Ok(())
            }),
            stats: false
        });
        async_complete_task(&objects.base.commander,&objects.base.messages,handle, |e| (e,false));
    }
    
    pub(super) fn set_carriages(&mut self, events: &mut CarriageEvents) {
        self.0.lock().unwrap().set_carriages(events);
    }
}
