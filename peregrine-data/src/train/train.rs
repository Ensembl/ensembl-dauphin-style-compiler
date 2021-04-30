use std::sync::{ Arc, Mutex };
use std::fmt;
use crate::api::{ PeregrineCore, CarriageSpeed, MessageSender };
use crate::core::{ Layout, Scale };
use super::carriage::Carriage;
use super::carriageset::CarriageSet;
use super::carriageevent::CarriageEvents;
use crate::run::{ add_task, async_complete_task };
use crate::util::message::DataMessage;
use crate::PgCommanderTaskSpec;
use peregrine_message::{ Reporter };

#[derive(Clone,Debug,Hash,PartialEq,Eq)]
pub struct TrainId {
    layout: Layout,
    scale: Scale
}

impl fmt::Display for TrainId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
    broken: bool,
    data_ready: bool,
    active: Option<u32>,
    id: TrainId,
    position: f64,
    max: Option<u64>,
    carriages: Option<CarriageSet>,
    messages: MessageSender
}

impl TrainData {
    fn new(id: &TrainId, carriage_event: &mut CarriageEvents, position: f64, messages: &MessageSender, reporter: &Reporter<DataMessage>) -> TrainData {
        let mut out = TrainData {
            broken: false,
            data_ready: false,
            active: None,
            id: id.clone(),
            position,
            carriages: Some(CarriageSet::new()),
            max: None,
            messages: messages.clone()
        };
        out.set_position(carriage_event,position,reporter);
        out
    }

    fn set_active(&mut self, carriage_event: &mut CarriageEvents, index: u32, quick: bool, reporter: &Reporter<DataMessage>) {
        if self.active != Some(index) {
            let speed = if quick { CarriageSpeed::Quick } else { CarriageSpeed::Slow };
            carriage_event.transition(index,self.max.unwrap(),speed,reporter);
        }
        self.active = Some(index);
    }

    fn set_inactive(&mut self) {
        self.active = None;
    }

    fn id(&self) -> &TrainId { &self.id }
    fn train_ready(&self) -> bool { self.data_ready && self.max.is_some() }
    fn is_broken(&self) -> bool { self.broken }

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
    fn set_position(&mut self, carriage_event: &mut CarriageEvents, position: f64, reporter: &Reporter<DataMessage>) {
        self.position = position;
        let carriage = self.id.scale.carriage(position);
        let carriages = CarriageSet::new_using(&self.id,carriage_event,carriage,self.carriages.take().unwrap(),&self.messages,reporter);
        self.carriages = Some(carriages);
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
            if let Some(reporter) = carriages.depend() {
                if let Some(index) = self.active {
                    events.set_carriages(&self.carriages(),index,&reporter);
                }
            }
        }
    }
}

// XXX circular chroms
#[derive(Clone)]
pub struct Train(Arc<Mutex<TrainData>>,MessageSender);

impl Train {
    pub(super) fn new(id: &TrainId, carriage_event: &mut CarriageEvents, position: f64, messages: &MessageSender, reporter: &Reporter<DataMessage>) -> Train {
        let out = Train(Arc::new(Mutex::new(TrainData::new(id,carriage_event,position,&messages,reporter))),messages.clone());
        carriage_event.train(&out,reporter);
        out
    }

    pub fn id(&self) -> TrainId { self.0.lock().unwrap().id().clone() }
    pub(super) fn train_ready(&self) -> bool { self.0.lock().unwrap().train_ready() }
    pub(super) fn train_broken(&self) -> bool { self.0.lock().unwrap().is_broken() }

    pub(super) fn set_active(&mut self, carriage_event: &mut CarriageEvents, index: u32, quick: bool, reporter: &Reporter<DataMessage>) {
        self.0.lock().unwrap().set_active(carriage_event,index,quick,reporter);
    }

    pub(super) fn set_inactive(&mut self) {
        self.0.lock().unwrap().set_inactive();
    }

    pub(super) fn set_position(&self, carriage_event: &mut CarriageEvents, position: f64, reporter: &Reporter<DataMessage>) {
        self.0.lock().unwrap().set_position(carriage_event,position,reporter);
    }

    pub(super) fn compatible_with(&self, other: &Train) -> bool {
        self.id().layout().stick() == other.id().layout().stick()
    }

    pub(super) fn maybe_ready(&mut self) {
        self.0.lock().unwrap().maybe_ready();
    }

    async fn find_max(&self, data: &mut PeregrineCore) -> Result<u64,DataMessage> {
        Ok(data.agent_store.stick_store().await.get(&self.id().layout().stick()).await?.size())
    }

    fn set_max(&self, max: Result<u64,DataMessage>) {
        self.0.lock().unwrap().set_max(max);
    }
    
    pub(super) fn run_find_max(&self, objects: &mut PeregrineCore, reporter: &Reporter<DataMessage>) {
        let self2 = self.clone();
        let mut objects2 = objects.clone();
        let reporter = reporter.clone();
        let handle = add_task(&objects.base.commander,PgCommanderTaskSpec {
            name: format!("max finder"),
            prio: 1,
            slot: None,
            timeout: None,
            task: Box::pin(async move {
                let max = self2.find_max(&mut objects2).await;
                if let Err(e) = &max { 
                    reporter.error(e.clone());
                    objects2.base.messages.send(e.clone());
                }
                self2.set_max(max);
                objects2.train_set.clone().update_trains(&mut objects2);
                Ok(())
            })
        });
        async_complete_task(&objects.base.commander,&objects.base.messages,handle, |e| (e,false));
    }
    
    pub(super) fn set_carriages(&mut self, events: &mut CarriageEvents) {
        self.0.lock().unwrap().set_carriages(events);
    }
}
