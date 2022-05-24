use std::sync::{Arc, Mutex};
use peregrine_toolkit::lock;
use peregrine_toolkit_async::sync::{blocker::Blocker};
use crate::{DataMessage, ShapeStore, PeregrineCoreBase, Viewport, StickStore};
use super::{trainset::TrainSet, railwaydatatasks::RailwayDataTasks};

#[derive(Clone)]
pub struct Railway {
    train_set: Arc<Mutex<TrainSet>>,
    carriage_loader: RailwayDataTasks
}

impl Railway {
    pub fn new(base: &PeregrineCoreBase, result_store: &ShapeStore, stick_store: &StickStore, visual_blocker: &Blocker) -> Railway {
        let mut carriage_loader = RailwayDataTasks::new(base,result_store,&stick_store,&base.redraw_needed);
        let railway = Railway {
            train_set: Arc::new(Mutex::new(TrainSet::new(base,result_store,visual_blocker,&base.redraw_needed.clone(),&carriage_loader))),
            carriage_loader: carriage_loader.clone(),
        };
        carriage_loader.set_railway(&railway);
        railway
    }

    pub(crate) fn ping(&self) {
        lock!(self.train_set).ping();
        self.carriage_loader.load();
    }

    pub fn set(&self, viewport: &Viewport) -> Result<(),DataMessage> {
        if viewport.ready() {
            lock!(self.train_set).set_position(viewport)?;
        }
        self.carriage_loader.load();
        Ok(())
    }

    pub fn transition_complete(&self, base: &mut PeregrineCoreBase) {
        lock!(self.train_set).transition_complete();
        self.carriage_loader.load();
    }

    pub fn set_sketchy(&self, yn: bool) -> Result<(),DataMessage> {
        lock!(self.train_set).set_sketchy(yn)?;
        self.carriage_loader.load();
        Ok(())
    }

    pub fn invalidate(&self) -> Result<(),DataMessage> {
        lock!(self.train_set).invalidate()?;
        self.carriage_loader.load();
        Ok(())
    }
}
