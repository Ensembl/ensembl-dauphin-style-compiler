use std::sync::{Arc, Mutex};
use peregrine_toolkit::{lock, sync::{blocker::Blocker, needed::Needed}};
use crate::{Carriage, DataMessage, ShapeStore, PeregrineCore, PeregrineCoreBase, PgCommanderTaskSpec, Viewport, add_task, api::MessageSender, async_complete_task, shapeload::loadshapes::LoadMode};
use super::{railwayevent::RailwayEvents, trainset::TrainSet};

#[derive(Clone)]
pub struct Railway {
    try_lifecycle: Needed,
    train_set: Arc<Mutex<TrainSet>>,
    messages: MessageSender
}

impl Railway {
    pub fn new(base: &PeregrineCoreBase,result_store: &ShapeStore, visual_blocker: &Blocker) -> Railway {
        let try_lifecycle = Needed::new();
        Railway {
            try_lifecycle: try_lifecycle.clone(),
            train_set: Arc::new(Mutex::new(TrainSet::new(base,result_store,visual_blocker,&try_lifecycle))),
            messages: base.messages.clone()
        }
    }

    async fn load_carriages(&self, objects: &mut PeregrineCore, mut carriages: Vec<Carriage>) {
        let mut loads = vec![];
        let commander= objects.base.commander.clone();
        for carriage in carriages.drain(..) {
            let objects2 = objects.clone();
            let handle = add_task(&commander,PgCommanderTaskSpec {
                    name: format!("single carriage loader"),
                    prio: 1,
                    slot: None,
                    timeout: None,
                    task: Box::pin(async move {
                        let mut carriage = carriage;
                        let r = carriage.load(&objects2.base,&objects2.agent_store.lane_store,LoadMode::RealTime).await;
                        if r.is_ok() && !carriage.is_moribund() {
                            lock!(objects2.base.integration).create_carriage(&carriage);
                        }
                        Ok(r)        
                    }),
                    stats: false
                });
            loads.push(handle);
        }
        for future in loads {
            let r = future.finish_future().await;
            let r = future.take_result().unwrap();
            if let Err(e) = r {
                self.messages.send(e.clone());
            }
        }
    }

    fn run_events(&self, mut events: RailwayEvents, objects: &mut PeregrineCore) {
        let loads = events.run_events(objects);
        if loads.len() > 0 {
           self.run_load_carriages(objects,loads);
        }
        lock!(self.train_set).update_dependents();
    }

    pub(super) fn move_and_lifecycle_trains(&mut self, objects: &mut PeregrineCore) {
        let events = lock!(self.train_set).move_and_lifecycle_trains();
        self.run_events(events,objects);
    }

    fn run_load_carriages(&self, objects: &mut PeregrineCore, loads: Vec<Carriage>) {
        let mut self2 = self.clone();
        let mut objects2 = objects.clone();
        let loads = loads.clone();
        let handle = add_task(&objects.base.commander,PgCommanderTaskSpec {
            name: format!("carriage loader"),
            prio: 1,
            slot: None,
            timeout: None,
            task: Box::pin(async move {
                self2.load_carriages(&mut objects2,loads).await;
                self2.move_and_lifecycle_trains(&mut objects2);
                Ok(())
            }),
            stats: false
        });
        async_complete_task(&objects.base.commander, &objects.base.messages,handle,|e| (e,false));
    }

    pub fn set(&self, objects: &mut PeregrineCore, viewport: &Viewport) -> Result<(),DataMessage> {
        let mut events = RailwayEvents::new();
        if viewport.ready() {
            lock!(self.train_set).set_position(&mut events,viewport)?;
        }
        events.draw_notify_viewport(viewport,true);
        self.run_events(events,objects);
        Ok(())
    }

    pub fn transition_complete(&self, objects: &mut PeregrineCore) {
        let mut events = RailwayEvents::new();
        lock!(self.train_set).transition_complete(&mut events);
        self.run_events(events,objects);
    }

    pub fn try_lifecycle_trains(&self, objects: &mut PeregrineCore) {
        if self.try_lifecycle.is_needed() {
            let mut train_set = lock!(self.train_set);
            let events = train_set.move_and_lifecycle_trains();
            drop(train_set);
            self.run_events(events,objects);
        }
    }

    pub fn set_sketchy(&self, objects: &mut PeregrineCore, yn: bool) -> Result<(),DataMessage> {
        let mut train_set = lock!(self.train_set);
        let events = train_set.set_sketchy(yn)?;
        drop(train_set);
        self.run_events(events,objects);
        Ok(())
    }

    pub fn invalidate(&self, objects: &mut PeregrineCore) -> Result<(),DataMessage> {
        let mut train_set = lock!(self.train_set);
        let events = train_set.invalidate()?;
        drop(train_set);
        self.run_events(events,objects);
        Ok(())
    }
}
