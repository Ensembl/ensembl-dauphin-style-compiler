use std::sync::{Arc, Mutex};

use peregrine_toolkit::{sync::needed::Needed, puzzle::AnswerAllocator};

use crate::{shapeload::carriageprocess::CarriageProcess, allotment::core::trainstate::{TrainStateSpec, TrainState3}};

use super::{drawingcarriagemanager::DrawingCarriageCreator, railwaydatatasks::RailwayDataTasks, graphics::Graphics, slider::SliderActions, carriageset::CarriageSetConstant};

pub(super) struct CarriageProcessManager {
    ping_needed: Needed,
    mute: bool,
    active: bool,
    constant: Arc<CarriageSetConstant>,
    railway_data_tasks: RailwayDataTasks, 
    train_state_spec: TrainStateSpec,
    graphics: Graphics
}

impl CarriageProcessManager {
    pub(super) fn new(ping_needed: &Needed, constant: &Arc<CarriageSetConstant>, 
           railway_data_tasks: &RailwayDataTasks, answer_allocator: &Arc<Mutex<AnswerAllocator>>,
            graphics: &Graphics) -> CarriageProcessManager {
        CarriageProcessManager {
            ping_needed: ping_needed.clone(),
            mute: false,
            active: false,
            constant: constant.clone(),
            graphics: graphics.clone(),
            railway_data_tasks: railway_data_tasks.clone(),
            train_state_spec: TrainStateSpec::new(answer_allocator)
        }
    }

    fn state_updated(&mut self) {
        if !self.mute && self.active {
            self.graphics.set_playing_field(self.state().playing_field());
            self.graphics.set_metadata(self.state().metadata());
        }
    }

    pub(super) fn mute(&mut self, yn: bool) {
        self.mute = yn;
        if !self.mute {
            self.state_updated()
        }
    }

    pub(super) fn active(&mut self) {
        self.active = true;
        self.state_updated();
    }

    pub(super) fn state(&self) -> TrainState3 { self.train_state_spec.spec() }
}

impl SliderActions<u64,CarriageProcess,DrawingCarriageCreator> for CarriageProcessManager {
    fn ctor(&mut self, index: &u64) -> CarriageProcess {
        let new_carriage = self.constant.new_unloaded_carriage(*index);
        self.railway_data_tasks.add_carriage(&new_carriage);
        new_carriage
    }

    fn dtor(&mut self, index: &u64, _item: DrawingCarriageCreator) {
        self.train_state_spec.remove(*index);
        self.state_updated();
    }

    fn init(&mut self, index: &u64, item: &mut CarriageProcess) -> Option<DrawingCarriageCreator> {
        item.get_shapes2().map(|shapes| {
            self.train_state_spec.add(*index,&shapes.spec().ok().unwrap()); // XXX errors
            self.state_updated();
            self.ping_needed.set(); /* Need to call ping in case dc are ready */
            DrawingCarriageCreator::new(
                shapes.clone(),
                item.extent().clone(),                
                self.ping_needed.clone()
            )
        })
    }
}
