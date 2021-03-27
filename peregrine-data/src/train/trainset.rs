use std::sync::{ Arc, Mutex };
use crate::{PeregrineCoreBase, PgCommanderTaskSpec};
use crate::api::{PeregrineCore, MessageSender };
use crate::core::{ Scale, Viewport };
use super::train::{ Train, TrainId };
use super::carriage::Carriage;
use super::carriageevent::CarriageEvents;
use blackbox::{ blackbox_time, blackbox_log };
use crate::run::{ add_task, async_complete_task };
use crate::util::message::DataMessage;
use peregrine_message::{ Instigator, Reporter };

/* current: train currently being displayed, if any. During transition, the outgoing train.
 * future: incoming train during transition.
 * wanted: train to be displayed when complete and when transitions are done.
 */

pub struct TrainSetData {
    current: Option<Train>,
    future: Option<Train>,
    wanted: Option<(Train,Reporter<DataMessage>)>,
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
        if self.wanted.as_ref().map(|x| x.0.train_ready() && !x.0.train_broken()).unwrap_or(false) && self.future.is_none() {
            if let Some((mut wanted,wanted_reporter)) = self.wanted.take() {
                blackbox_log!("uiapi","TrainSet.promote_future() wanted -> future");
                let quick = self.current.as_ref().map(|x| x.compatible_with(&wanted)).unwrap_or(true);
                wanted.set_active(events,self.next_activation,quick,&wanted_reporter);
                self.next_activation += 1;
                self.future = Some(wanted);
            }
        }
    }

    fn new_wanted(&mut self, events: &mut CarriageEvents, train_id: &TrainId, position: f64, reporter: &Reporter<DataMessage>) {
        blackbox_log!("uiapi","TrainSet.new_wanted()");
        self.wanted = Some((Train::new(train_id,events,position,&self.messages,reporter),reporter.clone()));
    }

    fn quiescent(&self) -> Option<&Train> {
        /* The quiescent train is the train which, barring this and any future changes will ultimately be displayed. */
        if let Some((wanted,_)) = &self.wanted {
            Some(wanted)
        } else if let Some(future) = &self.future {
            Some(future)
        } else if let Some(current) = &self.current {
            Some(current)
        } else {
            None
        }
    }

    fn maybe_new_wanted(&mut self, events: &mut CarriageEvents, viewport: &Viewport, reporter: &Reporter<DataMessage>) -> Result<(),DataMessage> {
        let train_id = TrainId::new(viewport.layout(),&Scale::new_bp_per_screen(viewport.bp_per_screen()?));
        let mut new_target_needed = true;
        if let Some(quiescent) = self.quiescent() {
            if quiescent.id() == train_id {
                new_target_needed = false;
            }
        }
        if new_target_needed {
            self.new_wanted(events,&train_id,viewport.position()?,reporter);
        }
        Ok(())
    }

    fn set_train_position(&self, events: &mut CarriageEvents, train: Option<&Train>, viewport: &Viewport, reporter: &Reporter<DataMessage>) -> Result<(),DataMessage> {
        if let Some(train) = train {
            if viewport.layout().stick() == train.id().layout().stick() {
                train.set_position(&mut events.clone(),viewport.position()?,reporter);
            }
        }
        Ok(())
    }

    fn set(&mut self, events: &mut CarriageEvents, viewport: &Viewport, reporter: Instigator<DataMessage>) -> Result<(),DataMessage> {
        let reporter = reporter.to_reporter();
        if !viewport.ready() { return Ok(()); }
        self.maybe_new_wanted(events,viewport,&reporter)?;
        self.set_train_position(events,self.wanted.as_ref().map(|x| &x.0),viewport,&reporter)?;
        self.set_train_position(events,self.future.as_ref(),viewport,&reporter)?;
        self.set_train_position(events,self.current.as_ref(),viewport,&reporter)?;
        self.promote(events);
        Ok(())
    }

    fn transition_complete(&mut self, events: &mut CarriageEvents) {
        blackbox_log!("uiapi","TrainSet.promote_future() future -> current");
        if let Some(mut current) = self.current.take() {
            current.set_inactive();
        }
        self.current = self.future.take();
        self.promote(events);
    }

    fn update_trains(&mut self) -> CarriageEvents {
        let mut events = CarriageEvents::new();
        if let Some(wanted) = &mut self.wanted {
            wanted.0.maybe_ready();
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

    async fn load_carriages(&self, objects: &mut PeregrineCore, carriages: &[(Carriage,Reporter<DataMessage>)]) {
        let mut loads = vec![];
        for (carriage,reporter) in carriages {
            loads.push((carriage.load(&objects),reporter));
        }
        for (future,reporter) in loads {
            let r = future.await;
            if let Err(e) = r {
                self.messages.send(e.clone());
                reporter.error(e.clone());
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

    pub(super) fn run_load_carriages(&self, objects: &mut PeregrineCore, loads: Vec<(Carriage,Reporter<DataMessage>)>) {
        let mut self2 = self.clone();
        let mut objects2 = objects.clone();
        let loads = loads.clone();
        let handle = add_task(&objects.base.commander,PgCommanderTaskSpec {
            name: format!("carriage loader"),
            prio: 1,
            slot: None,
            timeout: None,
            task: Box::pin(async move {
                self2.load_carriages(&mut objects2,&loads).await;
                self2.update_trains(&mut objects2);
                Ok(())
            })
        });
        async_complete_task(&objects.base.commander, &objects.base.messages,handle,|e| (e,false));
    }

    pub fn set(&self, objects: &mut PeregrineCore, viewport: &Viewport, reporter: Instigator<DataMessage>) {
        blackbox_time!("train","trainset-set",{
            if viewport.layout().stick().is_some() {
                let mut events = CarriageEvents::new();
                self.state.lock().unwrap().set(&mut events,viewport,reporter);
                self.run(events,objects);
            }
        });
    }

    pub fn transition_complete(&self, objects: &mut PeregrineCore) {
        let mut events = CarriageEvents::new();
        self.state.lock().unwrap().transition_complete(&mut events);
        self.run(events,objects);
    }
}
