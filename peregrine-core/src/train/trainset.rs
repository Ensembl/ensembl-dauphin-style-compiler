use std::sync::{ Arc, Mutex };
use crate::PgCommanderTaskSpec;
use crate::api::PeregrineObjects;
use crate::core::{ Scale, Viewport };
use super::train::{ Train, TrainId };
use super::carriage::Carriage;
use super::carriageevent::CarriageEvents;

/* current: train currently being displayed, if any. During transition, the outgoing train.
 * future: incoming train during transition.
 * wanted: train to be displayed when complete and when transitions are done.
 */

pub struct TrainSetData {
    current: Option<Train>,
    future: Option<Train>,
    wanted: Option<Train>,
    next_activation: u32
}

impl TrainSetData {
    pub fn new() -> TrainSetData {
        TrainSetData {
            current: None,
            future: None,
            wanted: None,
            next_activation: 0
        }
    }

    fn promote_future(&mut self) {
        if self.current.is_none() {
            self.current = self.future.take();
        }
    }

    fn promote_wanted(&mut self, events: &mut CarriageEvents) {
        if let Some(mut wanted) = self.wanted.take() {
            if wanted.train_ready() && self.future.is_none() {
                let quick = self.current.as_ref().map(|x| x.compatible_with(&wanted)).unwrap_or(true);
                wanted.set_active(events,self.next_activation,quick);
                self.next_activation += 1;
                self.future = Some(wanted);
            }
        }
    }

    fn promote(&mut self, events: &mut CarriageEvents) {
        self.promote_future();
        self.promote_wanted(events);
        self.promote_future();
    }

    fn new_wanted(&mut self, events: &mut CarriageEvents, train_id: &TrainId, position: f64) {
        self.wanted = Some(Train::new(train_id,events,position));
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
        let train_id = TrainId::new(viewport.layout(),&Scale::new_for_numeric(viewport.scale()));
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

    pub fn transition_complete(&mut self, events: &mut CarriageEvents) {
        if let Some(mut current) = self.current.take() {
            current.set_inactive();
        }
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
    pub fn new() -> TrainSet {
        TrainSet(Arc::new(Mutex::new(TrainSetData::new())))
    }

    async fn load_carriages(&self, objects: &mut PeregrineObjects, carriages: &[Carriage]) {
        let mut loads = vec![];
        for carriage in carriages {
            loads.push((carriage,carriage.load(&objects)));
        }
        for carriage in carriages {
            carriage.load(objects).await;
        }
    }
    
    fn maybe_ready(&mut self, objects: &mut PeregrineObjects) {
        let mut events = CarriageEvents::new();
        self.0.lock().unwrap().maybe_ready(&mut events);
        events.run(objects);
    }

    fn maybe_notify_ui(&mut self, objects: &mut PeregrineObjects) {
        let mut events = CarriageEvents::new();
        self.0.lock().unwrap().maybe_notify_ui(&mut events);
        events.run(objects);
    }

    pub(super) fn poll(&mut self, objects: &mut PeregrineObjects) {
        self.maybe_ready(objects);
        self.maybe_notify_ui(objects);
    }

    pub(super) fn run_load_carriages(&self, objects: &mut PeregrineObjects, carriages: Vec<Carriage>) {
        let mut self2 = self.clone();
        let mut objects2 = objects.clone();
        let carriages = carriages.clone();
        objects.commander.add_task(PgCommanderTaskSpec {
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

    pub fn set(&self, objects: &mut PeregrineObjects, viewport: &Viewport) {
        let mut events = CarriageEvents::new();
        if viewport.layout().stick().is_some() {
            self.0.lock().unwrap().set(&mut events,viewport);
        }
        events.run(objects);
    }

    pub fn transition_complete(&self, objects: &mut PeregrineObjects) {
        let mut events = CarriageEvents::new();
        self.0.lock().unwrap().transition_complete(&mut events);
        events.run(objects);
    }
}
