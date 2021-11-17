use std::sync::{Arc, Mutex};

use peregrine_toolkit::sync::blocker::Blocker;

use crate::{Carriage, DataMessage, LaneStore, PeregrineCore, PeregrineCoreBase, PgCommanderTaskSpec, Viewport, add_task, api::MessageSender, async_complete_task, lane::shapeloader::LoadMode};

use super::{railwayevent::RailwayEvents, trainset::TrainSet};


#[derive(Clone)]
pub struct Railway {
    state: Arc<Mutex<TrainSet>>,
    messages: MessageSender
}

impl Railway {
    pub fn new(base: &PeregrineCoreBase,result_store: &LaneStore, visual_blocker: &Blocker) -> Railway {
        Railway {
            state: Arc::new(Mutex::new(TrainSet::new(base,result_store,visual_blocker))),
            messages: base.messages.clone()
        }
    }

    async fn load_carriages(&self, objects: &mut PeregrineCore, carriages: &mut [Carriage]) {
        let mut loads = vec![];
        for carriage in carriages {
            loads.push(carriage.load(&objects.base,&objects.agent_store.lane_store,LoadMode::RealTime));
        }
        for future in loads {
            let r = future.await;
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
        self.state.lock().unwrap().update_dependents();
    }

    pub(super) fn move_and_lifecycle_trains(&mut self, objects: &mut PeregrineCore) {
        let events = self.state.lock().unwrap().move_and_lifecycle_trains();
        self.run_events(events,objects);
    }

    fn run_load_carriages(&self, objects: &mut PeregrineCore, loads: Vec<Carriage>) {
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
            self.state.lock().unwrap().set_position(&mut events,viewport)?;
        }
        events.draw_notify_viewport(viewport,true);
        self.run_events(events,objects);
        Ok(())
    }

    pub fn transition_complete(&self, objects: &mut PeregrineCore) {
        let mut events = RailwayEvents::new();
        self.state.lock().unwrap().transition_complete(&mut events);
        self.run_events(events,objects);
    }
}
