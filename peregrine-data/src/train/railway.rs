use std::sync::{Arc, Mutex};
use peregrine_toolkit::{lock, sync::{blocker::Blocker, needed::Needed}, log};
use crate::{DataMessage, ShapeStore, PeregrineCore, PeregrineCoreBase, PgCommanderTaskSpec, Viewport, add_task, api::MessageSender, async_complete_task, shapeload::{loadshapes::LoadMode, carriageprocess::CarriageProcess}, StickStore};
use super::{railwayevent::RailwayEvents, trainset::TrainSet, railwaydatatasks::RailwayDataTasks};

#[derive(Clone)]
pub struct Railway {
    try_lifecycle: Needed,
    train_set: Arc<Mutex<TrainSet>>,
    carriage_loader: RailwayDataTasks
}

impl Railway {
    pub fn new(base: &PeregrineCoreBase, result_store: &ShapeStore, stick_store: &StickStore, visual_blocker: &Blocker) -> Railway {
        log!("A");
        let try_lifecycle = Needed::new();
        let mut carriage_loader = RailwayDataTasks::new(base,result_store,&stick_store,&try_lifecycle);
        log!("new()");
        let railway = Railway {
            try_lifecycle: try_lifecycle.clone(),
            train_set: Arc::new(Mutex::new(TrainSet::new(base,result_store,visual_blocker,&try_lifecycle))),
            carriage_loader: carriage_loader.clone(),
        };
        log!("set railway");
        carriage_loader.set_railway(&railway);
        railway
    }

    fn run_events(&self, mut events: RailwayEvents, base: &mut PeregrineCoreBase) {
        events.run_events(base,&self.carriage_loader);
        self.carriage_loader.load();
        lock!(self.train_set).update_dependents();
    }

    pub(super) fn move_and_lifecycle_trains(&self, base: &mut PeregrineCoreBase) {
        let events = lock!(self.train_set).move_and_lifecycle_trains(&self.carriage_loader);
        self.run_events(events,base);
        self.carriage_loader.load();
    }

    pub fn set(&self, base: &mut PeregrineCoreBase, viewport: &Viewport) -> Result<(),DataMessage> {
        let mut events = RailwayEvents::new(&self.try_lifecycle);
        if viewport.ready() {
            lock!(self.train_set).set_position(&mut events,&self.carriage_loader,viewport)?;
        }
        events.draw_notify_viewport(viewport,true);
        self.run_events(events,base);
        self.carriage_loader.load();
        Ok(())
    }

    pub fn transition_complete(&self, base: &mut PeregrineCoreBase) {
        let mut events = RailwayEvents::new(&self.try_lifecycle);
        lock!(self.train_set).transition_complete(&mut events,&self.carriage_loader);
        self.run_events(events,base);
        self.carriage_loader.load();
    }

    pub fn try_lifecycle_trains(&self, base: &mut PeregrineCoreBase) {
        if self.try_lifecycle.is_needed() {
            let mut train_set = lock!(self.train_set);
            let events = train_set.move_and_lifecycle_trains(&self.carriage_loader);
            drop(train_set);
            self.run_events(events,base);
            self.carriage_loader.load();
        }
    }

    pub fn set_sketchy(&self, base: &mut PeregrineCoreBase, yn: bool) -> Result<(),DataMessage> {
        let mut train_set = lock!(self.train_set);
        let events = train_set.set_sketchy(&self.carriage_loader,yn)?;
        drop(train_set);
        self.run_events(events,base);
        self.carriage_loader.load();
        Ok(())
    }

    pub fn invalidate(&self, base: &mut PeregrineCoreBase) -> Result<(),DataMessage> {
        let mut train_set = lock!(self.train_set);
        let events = train_set.invalidate(&self.carriage_loader)?;
        drop(train_set);
        self.run_events(events,base);
        self.carriage_loader.load();
        Ok(())
    }
}
