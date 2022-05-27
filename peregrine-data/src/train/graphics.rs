use std::{sync::{Arc, Mutex}, collections::{HashMap}};
use peregrine_toolkit::lock;
use crate::{PeregrineIntegration, allotment::{globals::{playingfield::{GlobalPlayingField, PlayingField}, allotmentmetadata::GlobalAllotmentMetadata} }, CarriageSpeed, Viewport, api::TrainIdentity };
use super::drawingcarriage::DrawingCarriage;

struct GraphicsDropper {
    state: Arc<Mutex<GraphicsState>>,
    integration: Arc<Mutex<Box<dyn PeregrineIntegration>>>,
}

struct GraphicsState {
    trains: HashMap<TrainIdentity,i32>, // create&destroy trains as needed
    playing_field: Option<GlobalPlayingField>, // don't repeat ourselves
    metadata: Option<GlobalAllotmentMetadata>, // don't repeat ourcelves, ;-)
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
            metadata: None    
        }));
        Graphics {
            dropper: Arc::new(GraphicsDropper{ state: state.clone(), integration: integration.clone() }),
            integration,
            state
        }
    }

    fn upate_train(&mut self, dc: &DrawingCarriage, delta: i32) {
        let mut state = lock!(self.state);
        let train = dc.train_identity();
        let value = state.trains.entry(train.clone()).or_insert(0);
        if *value == 0 {
            lock!(self.integration).create_train(&train);
        }
        *value += delta;
        if *value == 0 {
            lock!(self.integration).drop_train(&train);
        }
    }

    pub(super) fn create_carriage(&mut self, dc: &DrawingCarriage) {
        self.upate_train(dc,1);
        lock!(self.integration).create_carriage(dc);
    }

    pub(super) fn drop_carriage(&mut self, dc: &DrawingCarriage) {
        lock!(self.integration).drop_carriage(dc);
        self.upate_train(dc,-1);
    }

    pub(super) fn set_carriages(&self, train_identity: &TrainIdentity, carriages: &[DrawingCarriage]) {
        let state = lock!(self.state);
        if !state.trains.contains_key(train_identity) {
            panic!("set_carriages on dead train");
        }
        lock!(self.integration).set_carriages(&train_identity,carriages);
    }

    pub(super) fn start_transition(&self, train: &TrainIdentity, max: u64, speed: CarriageSpeed) {
        lock!(self.integration).start_transition(&train,max,speed);
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
