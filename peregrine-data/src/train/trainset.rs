use std::sync::{ Arc, Mutex };
use crate::{PeregrineCoreBase, PgCommanderTaskSpec};
use crate::api::{PeregrineCore, MessageSender };
use crate::core::{ Scale, Viewport };
use super::train::{ Train, TrainId };
use super::carriage::Carriage;
use super::carriageevent::CarriageEvents;
use crate::run::{ add_task, async_complete_task };
use crate::util::message::DataMessage;

/* current: train currently being displayed, if any. During transition, the outgoing train.
 * future: incoming train during transition.
 * wanted: train to be displayed when complete and when transitions are done.
 */

pub struct TrainSetData {
    current: Option<Train>,
    future: Option<Train>,
    wanted: Option<Train>,
    next_activation: u32,
    messages: MessageSender
}

impl TrainSetData {
    fn new(messages: &MessageSender) -> TrainSetData {
        TrainSetData {
            current: None,
            future: None,
            wanted: None,
            next_activation: 0,
            messages: messages.clone()
        }
    }

    fn promote(&mut self, events: &mut CarriageEvents) {
        if self.wanted.as_ref().map(|x| x.train_ready() && !x.train_broken()).unwrap_or(false) && self.future.is_none() {
            if let Some(mut wanted) = self.wanted.take() {
                let quick = self.current.as_ref().map(|x| x.compatible_with(&wanted)).unwrap_or(true);
                wanted.set_active(events,self.next_activation,quick);
                self.next_activation += 1;
                self.future = Some(wanted);
                self.notify_viewport(events);
            }
        }
    }

    fn new_wanted(&mut self, events: &mut CarriageEvents, train_id: &TrainId, viewport: &Viewport) -> Result<(),DataMessage> {
        self.wanted = Some(Train::new(train_id,events,viewport,&self.messages)?);
        Ok(())
    }

    fn quiescent(&self) -> Option<&Train> {
        /* The quiescent train is the train which, barring this and any future changes will ultimately be displayed. */
        if let Some(wanted) = &self.wanted {
            Some(wanted)
        } else if let Some(future) = &self.future {
            Some(future)
        } else if let Some(current) = &self.current {
            Some(current)
        } else {
            None
        }
    }

    fn notify_viewport(&self, events: &mut CarriageEvents) {
        if let Some(train) = self.future.as_ref().or_else(|| self.current.as_ref()) {
            events.notify_viewport(&train.viewport(),false);
        }
    }

    fn maybe_new_wanted(&mut self, events: &mut CarriageEvents, viewport: &Viewport) -> Result<(),DataMessage> {
        let train_id = TrainId::new(viewport.layout()?,&Scale::new_bp_per_screen(viewport.bp_per_screen()?));
        let mut new_target_needed = true;
        if let Some(quiescent) = self.quiescent() {
            if quiescent.id() == train_id {
                new_target_needed = false;
            }
        }
        if new_target_needed {
            self.new_wanted(events,&train_id,viewport)?;
        }
        Ok(())
    }

    fn set_train_position(&self, events: &mut CarriageEvents, train: Option<&Train>, viewport: &Viewport) -> Result<(),DataMessage> {
        if let Some(train) = train {
            if viewport.layout()?.stick() == train.id().layout().stick() {
                train.set_position(&mut events.clone(),viewport)?;
            }
        }
        Ok(())
    }

    fn set(&mut self, events: &mut CarriageEvents, viewport: &Viewport) -> Result<(),DataMessage> {
        if !viewport.ready() { return Ok(()); }
        self.maybe_new_wanted(events,viewport)?;
        self.set_train_position(events,self.wanted.as_ref(),viewport)?;
        self.set_train_position(events,self.future.as_ref(),viewport)?;
        self.set_train_position(events,self.current.as_ref(),viewport)?;
        self.promote(events);
        self.notify_viewport(events);
        Ok(())
    }

    fn transition_complete(&mut self, events: &mut CarriageEvents) {
        if let Some(mut current) = self.current.take() {
            current.set_inactive();
        }
        self.current = self.future.take();
        self.promote(events);
    }

    fn update_trains(&mut self) -> CarriageEvents {
        let mut events = CarriageEvents::new();
        if let Some(wanted) = &mut self.wanted {
            wanted.maybe_ready();
            self.promote(&mut events);
        }
        if let Some(train) = &mut self.future {
            train.set_carriages(&mut events);
        }
        if let Some(train) = &mut self.current {
            train.set_carriages(&mut events);
        }
        events
    }
}

#[derive(Clone)]
pub struct TrainSet {
    state: Arc<Mutex<TrainSetData>>,
    messages: MessageSender
}

impl TrainSet {
    pub fn new(base: &PeregrineCoreBase) -> TrainSet {
        TrainSet {
            state: Arc::new(Mutex::new(TrainSetData::new(&base.messages))),
            messages: base.messages.clone()
        }
    }

    async fn load_carriages(&self, objects: &mut PeregrineCore, carriages: &mut [Carriage]) {
        let mut loads = vec![];
        for carriage in carriages {
            loads.push(carriage.load(&objects));
        }
        for future in loads {
            let r = future.await;
            if let Err(e) = r {
                self.messages.send(e.clone());
            }
        }
    }

    fn run(&self, mut events: CarriageEvents, objects: &mut PeregrineCore) {
        let loads = events.run(objects);
        if loads.len() > 0 {
           self.run_load_carriages(objects,loads);
        }
    }

    pub(super) fn update_trains(&mut self, objects: &mut PeregrineCore) {
        let events = self.state.lock().unwrap().update_trains();
        self.run(events,objects);
    }

    pub(super) fn run_load_carriages(&self, objects: &mut PeregrineCore, loads: Vec<Carriage>) {
        let mut self2 = self.clone();
        let mut objects2 = objects.clone();
        let mut loads = loads.clone();
        let handle = add_task(&objects.base.commander,PgCommanderTaskSpec {
            name: format!("carriage loader"),
            prio: 1,
            slot: None,
            timeout: None,
            task: Box::pin(async move {
                self2.load_carriages(&mut objects2,&mut loads).await;
                self2.update_trains(&mut objects2);
                Ok(())
            }),
            stats: false
        });
        async_complete_task(&objects.base.commander, &objects.base.messages,handle,|e| (e,false));
    }

    pub fn set(&self, objects: &mut PeregrineCore, viewport: &Viewport) -> Result<(),DataMessage> {
        let mut events = CarriageEvents::new();
        if viewport.ready() {
            self.state.lock().unwrap().set(&mut events,viewport)?;
        }
        events.notify_viewport(viewport,true);
        self.run(events,objects);
        Ok(())
    }

    pub fn transition_complete(&self, objects: &mut PeregrineCore) {
        let mut events = CarriageEvents::new();
        self.state.lock().unwrap().transition_complete(&mut events);
        self.run(events,objects);
    }
}
