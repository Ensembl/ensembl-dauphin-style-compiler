use std::sync::{Arc, Mutex};
use peregrine_toolkit::{sync::needed::Needed, puzzle::AnswerAllocator};
use crate::{shapeload::carriageprocess::CarriageProcess, allotment::core::trainstate::{TrainStateSpec, TrainState3}};
use super::{drawingcarriagemanager::DrawingCarriageCreator, railwaydatatasks::RailwayDataTasks, graphics::Graphics, party::PartyActions, carriageset::CarriageSetConstant};

#[cfg(debug_trains)]
use peregrine_toolkit::debug_log;

pub(super) struct CarriageProcessManager {
    ping_needed: Needed,
    ready: bool,
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
            ready: false,
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

impl PartyActions<u64,CarriageProcess,DrawingCarriageCreator> for CarriageProcessManager {
    fn ctor(&mut self, index: &u64) -> CarriageProcess {
        let new_carriage = self.constant.new_unloaded_carriage(*index);
        #[cfg(debug_trains)] debug_log!("CP ctor ({})",new_carriage.extent().compact());
        self.railway_data_tasks.add_carriage(&new_carriage);
        new_carriage
    }

    fn dtor_pending(&mut self, index: &u64, _carriage: CarriageProcess) {
        #[cfg(debug_trains)] debug_log!("CP dtor_pending ({})",_carriage.extent().compact());
        self.train_state_spec.remove(*index);
        self.state_updated();
        self.ping_needed.set(); /* Need to call ping in case dc are ready */
    }

    fn dtor(&mut self, index: &u64, _carriage: DrawingCarriageCreator) {
        #[cfg(debug_trains)] debug_log!("CP dtor ({})",_carriage.extent().compact());
        self.train_state_spec.remove(*index);
        self.state_updated();
        self.ping_needed.set(); /* Need to call ping in case dc are ready */
    }

    fn init(&mut self, index: &u64, carriage: &mut CarriageProcess) -> Option<DrawingCarriageCreator> {
        carriage.get_shapes2().map(|shapes| {
            #[cfg(debug_trains)] debug_log!("CP init ({})",carriage.extent().compact());
            self.train_state_spec.add(*index,&shapes.spec().ok().unwrap()); // XXX errors
            self.state_updated();
            self.ping_needed.set(); /* Need to call ping in case dc are ready */
            DrawingCarriageCreator::new(
                shapes.clone(),
                carriage.extent().clone(),                
                self.ping_needed.clone()
            )
        })
    }

    fn quiet(&mut self, _items: &mut dyn Iterator<Item=(&u64,&DrawingCarriageCreator)>) {
        self.ready = true;
    }
}
