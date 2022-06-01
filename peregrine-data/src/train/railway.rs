use std::sync::{Arc, Mutex};
use peregrine_toolkit::lock;
use peregrine_toolkit_async::sync::{blocker::Blocker};
use crate::{DataMessage, ShapeStore, PeregrineCoreBase, Viewport };
use super::{trainset::TrainSet, railwaydatatasks::RailwayDataTasks};

#[derive(Clone)]
pub struct Railway {
    train_set: Arc<Mutex<TrainSet>>
}

impl Railway {
    pub(crate) fn new(base: &PeregrineCoreBase, result_store: &ShapeStore, visual_blocker: &Blocker, carriage_loader: &RailwayDataTasks) -> Railway {
        let railway = Railway {
            train_set: Arc::new(Mutex::new(TrainSet::new(base,result_store,visual_blocker,&base.redraw_needed.clone(),&carriage_loader))),
        };
        railway
    }

    pub(crate) fn ping(&self) {
        lock!(self.train_set).ping();
    }

    pub(crate) fn set(&self, viewport: &Viewport) -> Result<(),DataMessage> {
        if viewport.ready() {
            lock!(self.train_set).set_position(viewport)?;
        }
        Ok(())
    }

    pub(crate) fn transition_complete(&self) {
        lock!(self.train_set).transition_complete();
    }

    pub(crate) fn set_sketchy(&self, yn: bool) -> Result<(),DataMessage> {
        lock!(self.train_set).set_sketchy(yn)?;
        Ok(())
    }

    pub(crate) fn invalidate(&self) -> Result<(),DataMessage> {
        lock!(self.train_set).invalidate()?;
        Ok(())
    }
}
