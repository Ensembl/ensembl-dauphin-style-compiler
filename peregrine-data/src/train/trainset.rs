use std::sync::{ Arc, Mutex };
use crate::PgCommanderTaskSpec;
use crate::api::{PeregrineCore, MessageSender };
use crate::core::{ Scale, Viewport };
use super::train::{ Train, TrainId };
use super::carriage::Carriage;
use super::carriageevent::CarriageEvents;
use blackbox::{ blackbox_time, blackbox_log };
use crate::run::add_task;

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
        if self.wanted.as_ref().map(|x| x.train_ready()).unwrap_or(false) && self.future.is_none() {
            if let Some(mut wanted) = self.wanted.take() {
                blackbox_log!("uiapi","TrainSet.promote_future() wanted -> future");
                let quick = self.current.as_ref().map(|x| x.compatible_with(&wanted)).unwrap_or(true);
                wanted.set_active(events,self.next_activation,quick);
                self.next_activation += 1;
                self.future = Some(wanted);
            }
        }
    }

    fn new_wanted(&mut self, events: &mut CarriageEvents, train_id: &TrainId, position: f64) {
        blackbox_log!("uiapi","TrainSet.new_wanted()");
        self.wanted = Some(Train::new(train_id,events,position,&self.messages));
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

    fn maybe_update_target(&mut self, events: &mut CarriageEvents, viewport: &Viewport) {
        use web_sys::console;
        let train_id = TrainId::new(viewport.layout(),&Scale::new_bp_per_screen(viewport.bp_per_screen()));
        let mut new_target_needed = true;
        if let Some(quiescent) = self.quiescent() {
            if quiescent.id() == train_id {
                new_target_needed = false;
            }
        }
        if new_target_needed {
            self.new_wanted(events,&train_id,viewport.position());
        }
    }

    fn update_train(&self, events: &mut CarriageEvents, train: &Option<Train>, viewport: &Viewport) {
        if let Some(train) = train {
            if viewport.layout().stick() == train.id().layout().stick() {
                train.set_position(&mut events.clone(),viewport.position());
            }
        }
    }

    fn set(&mut self, events: &mut CarriageEvents, viewport: &Viewport) {
        self.maybe_update_target(events,viewport);
        self.update_train(events,&self.wanted,viewport);
        self.update_train(events,&self.future,viewport);
        self.update_train(events,&self.current,viewport);
        self.promote(events);
    }

    fn transition_complete(&mut self, events: &mut CarriageEvents) {
        blackbox_log!("uiapi","TrainSet.promote_future() future -> current");
        if let Some(mut current) = self.current.take() {
            current.set_inactive();
        }
        self.current = self.future.take();
        self.promote(events);
    }

    fn maybe_ready(&mut self, events: &mut CarriageEvents) {
        if let Some(wanted) = &mut self.wanted {
            wanted.maybe_ready();
            self.promote(events);
        }
    }

    fn maybe_notify_ui(&mut self, events: &mut CarriageEvents) {
        if let Some(train) = &mut self.future {
            train.maybe_notify_ui(events);
        }
        if let Some(train) = &mut self.current {
            train.maybe_notify_ui(events);
        }
    }
}

#[derive(Clone)]
pub struct TrainSet(Arc<Mutex<TrainSetData>>);

impl TrainSet {
    pub fn new(messages: &MessageSender) -> TrainSet {
        TrainSet(Arc::new(Mutex::new(TrainSetData::new(messages))))
    }

    async fn load_carriages(&self, objects: &mut PeregrineCore, carriages: &[Carriage]) {
        let mut loads = vec![];
        for carriage in carriages {
            ////console::log_1(&format!("TrainSet.load_carriage() carriage={:?}",carriage).into());
            loads.push((carriage,carriage.load(&objects)));
        }
        for carriage in carriages {
            carriage.load(objects).await;
        }
        //console::log_1(&format!("TrainSet.load_carriage() loaded!").into());
    }
    
    pub(super) fn poll(&mut self, objects: &mut PeregrineCore) {
        let mut events = CarriageEvents::new();
        self.0.lock().unwrap().maybe_ready(&mut events);
        self.0.lock().unwrap().maybe_notify_ui(&mut events);
        events.run(objects);
    }

    pub(super) fn run_load_carriages(&self, objects: &mut PeregrineCore, carriages: Vec<Carriage>) {
        ////console::log_1(&format!("TrainSet.run_load_carriages").into());
        let mut self2 = self.clone();
        let mut objects2 = objects.clone();
        let carriages = carriages.clone();
        add_task(&objects.commander,PgCommanderTaskSpec {
            name: format!("carriage loader"),
            prio: 1,
            slot: None,
            timeout: None,
            task: Box::pin(async move {
                self2.load_carriages(&mut objects2,&carriages).await;
                self2.poll(&mut objects2);
                Ok(())
            })
        });
    }

    pub fn set(&self, objects: &mut PeregrineCore, viewport: &Viewport) {
        blackbox_time!("train","trainset-set",{
            let mut events = CarriageEvents::new();
            if viewport.layout().stick().is_some() {
                self.0.lock().unwrap().set(&mut events,viewport);
            }
            events.run(objects);
        });
    }

    pub fn transition_complete(&self, objects: &mut PeregrineCore) {
        let mut events = CarriageEvents::new();
        self.0.lock().unwrap().transition_complete(&mut events);
        events.run(objects);
    }
}
