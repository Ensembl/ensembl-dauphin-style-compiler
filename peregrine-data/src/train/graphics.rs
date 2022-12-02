use std::{sync::{Arc, Mutex}, collections::{HashMap, VecDeque}};
use peregrine_toolkit::lock;
use crate::{PeregrineIntegration, CarriageSpeed, Viewport, api::TrainIdentity, Stick };
use super::drawing::drawingcarriage::DrawingCarriage;
use crate::globals::{playingfield::{GlobalPlayingField, PlayingField}, allotmentmetadata::GlobalAllotmentMetadata};

#[cfg(debug_trains)]
use peregrine_toolkit::log;

struct GraphicsDropper {
    state: Arc<Mutex<GraphicsState>>,
    integration: Arc<Mutex<Box<dyn PeregrineIntegration>>>,
}

#[derive(Clone)]
struct TransitionSpec {
    train: TrainIdentity,
    max: u64,
    speed: CarriageSpeed
}

impl TransitionSpec {
    fn new(train: &TrainIdentity, max: u64, speed: &CarriageSpeed) -> TransitionSpec {
        TransitionSpec {
            train: train.clone(),
            max,
            speed: speed.clone()
        }
    }

    fn go(&self, integration: &mut dyn PeregrineIntegration) {
        integration.start_transition(&self.train,self.max,self.speed.clone());
    }
}

struct DisplayedTrains {
    from: Option<TrainIdentity>,
    to: Option<TrainIdentity>,
    future: Option<TransitionSpec>,
    running: bool,
}

impl DisplayedTrains {
    fn new() -> DisplayedTrains {
        DisplayedTrains {
            from: None,
            to: None,
            future: None,
            running: false,
        }
    }
}

struct GraphicsState {
    trains: HashMap<TrainIdentity,i32>, // create&destroy trains as needed
    transition: DisplayedTrains,
    playing_field: Option<GlobalPlayingField>, // don't repeat ourselves
    metadata: Option<GlobalAllotmentMetadata>, // don't repeat ourselves, ;-)
}

#[derive(Clone)]
pub(crate) struct Graphics {
    /* Main way of contacting graphics */
    integration: Arc<Mutex<Box<dyn PeregrineIntegration>>>,
    /* API-local state */
    state: Arc<Mutex<GraphicsState>>,
    #[allow(unused)]
    dropper: Arc<GraphicsDropper>
}

impl Graphics {
    pub(crate) fn new(integration: &Arc<Mutex<Box<dyn PeregrineIntegration>>>) -> Graphics {
        let integration = integration.clone();
        let state = Arc::new(Mutex::new(GraphicsState {
            trains: HashMap::new(),
            playing_field: None,
            transition: DisplayedTrains::new(),
            metadata: None    
        }));
        Graphics {
            dropper: Arc::new(GraphicsDropper{ state: state.clone(), integration: integration.clone() }),
            integration,
            state
        }
    }

    fn upate_train(&mut self, train_identity: &TrainIdentity, delta: i32) {
        let mut state = lock!(self.state);
        let value = state.trains.entry(train_identity.clone()).or_insert(0);
        if *value == 0 {
            lock!(self.integration).create_train(&train_identity);
        }
        *value += delta;
        if *value == 0 {
            lock!(self.integration).drop_train(&train_identity);
        }
    }

    pub(super) fn create_carriage(&mut self, dc: &DrawingCarriage) {
        self.upate_train(dc.train_identity(),1);
        lock!(self.integration).create_carriage(dc);
    }

    pub(super) fn drop_carriage(&mut self, dc: &DrawingCarriage) {
        lock!(self.integration).drop_carriage(dc);
        self.upate_train(dc.train_identity(),-1);
    }

    pub(super) fn set_carriages(&self, train_identity: &TrainIdentity, carriages: &[DrawingCarriage]) {
        let state = lock!(self.state);
        if !state.trains.contains_key(train_identity) {
            panic!("set_carriages on dead train");
        }
        lock!(self.integration).set_carriages(&train_identity,carriages);
    }

    pub(super) fn start_transition(&mut self, train: &TrainIdentity, stick: &Stick, speed: CarriageSpeed) {
        self.upate_train(train,1);
        let mut state = lock!(self.state);
        let transition = TransitionSpec::new(train,stick.size(),&speed);
        if !state.transition.running {
            state.transition.running = true;
            state.transition.from = state.transition.to.take();
            state.transition.to = Some(train.clone());
            drop(state);
            #[cfg(debug_trains)]
            log!("start transition to train {:?}",train);
            transition.go(&mut **lock!(self.integration));
        } else {
            let old_target = state.transition.future.take();
            state.transition.future = Some(transition);
            drop(state);
            if let Some(old_target) = old_target {
                self.upate_train(&old_target.train,-1);
            }
        }
    }

    pub(super) fn transition_complete(&mut self) {
        let mut state = lock!(self.state);
        let done_train = state.transition.from.take();
        let next_train = if let Some(next_train) = state.transition.future.take() {
            state.transition.running = true;
            state.transition.from = state.transition.to.take();
            state.transition.to = Some(next_train.train.clone());
            Some(next_train)
        } else {
            state.transition.running = false;
            None
        };
        drop(state);
        if let Some(train) = done_train {
            #[cfg(debug_trains)]
            log!("end transition to train {:?}",train);    
            self.upate_train(&train,-1);
        }
        if let Some(train) = next_train {
            #[cfg(debug_trains)]
            log!("start transition to train {:?}",train.train);
            train.go(&mut **lock!(self.integration));
        }
    }

    pub(super) fn set_playing_field(&mut self, playing_field: &GlobalPlayingField) {
        let mut state = lock!(self.state);
        if let Some(old_playing_field) = &state.playing_field {
            if old_playing_field == playing_field { return; }
        }
        state.playing_field = Some(playing_field.clone());
        let playing_field = PlayingField::new(playing_field);
        drop(state);
        lock!(self.integration).set_playing_field(playing_field.clone());
    }

    pub(super) fn set_metadata(&mut self, metadata: &GlobalAllotmentMetadata) {
        let mut state = lock!(self.state);
        if let Some(old_metadata) = &state.metadata {
            if old_metadata == metadata { return; }
        }
        state.metadata = Some(metadata.clone());
        drop(state);
        lock!(self.integration).notify_allotment_metadata(metadata);
    }

    pub(super) fn notify_viewport(&self, viewport: &Viewport) {
        lock!(self.integration).notify_viewport(viewport);
    }
}

impl Drop for GraphicsDropper {
    fn drop(&mut self) {
        let state = lock!(self.state);
        let trains = state.trains.clone();
        for train in trains.keys() {
            lock!(self.integration).drop_train(&train);
        }
    }
}
