use std::{sync::{Arc, Mutex}, collections::{HashSet, HashMap}};

use peregrine_toolkit::{lock, log};

use crate::{PeregrineIntegration, TrainExtent, allotment::{transformers::transformertraits::GraphTransformer, core::playingfield::{GlobalPlayingField, PlayingField}}, CarriageSpeed, GlobalAllotmentMetadata, Viewport };

use super::drawingcarriage::DrawingCarriage2;

struct GraphicsDropper {
    trains: Arc<Mutex<HashMap<TrainExtent,i32>>>,
    integration: Arc<Mutex<Box<dyn PeregrineIntegration>>>,
}

#[derive(Clone)]
pub(crate) struct Graphics {
    trains: Arc<Mutex<HashMap<TrainExtent,i32>>>,
    integration: Arc<Mutex<Box<dyn PeregrineIntegration>>>,
    #[allow(unused)]
    dropper: Arc<GraphicsDropper>
}

impl Graphics {
    pub(crate) fn new(integration: &Arc<Mutex<Box<dyn PeregrineIntegration>>>) -> Graphics {
        let trains = Arc::new(Mutex::new(HashMap::new()));
        let integration = integration.clone();
        Graphics {
            dropper: Arc::new(GraphicsDropper{ trains: trains.clone(), integration: integration.clone() }),
            trains, integration
        }
    }

    fn upate_train(&mut self, dc: &DrawingCarriage2, delta: i32) {
        let mut trains = lock!(self.trains);
        let train = dc.extent().train();
        let value = trains.entry(train.clone()).or_insert(0);
        if *value == 0 {
            log!("gl/create train {:?}",train);
            lock!(self.integration).create_train(train);
        }
        *value += delta;
        if *value == 0 {
            lock!(self.integration).drop_train(train);
        }
    }

    pub(super) fn create_carriage(&mut self, dc: &DrawingCarriage2) {
        self.upate_train(dc,1);
        log!("gl/create carriage {:?}",dc.extent().index());
        lock!(self.integration).create_carriage(dc);
    }

    pub(super) fn drop_carriage(&mut self, dc: &DrawingCarriage2) {
        log!("gl/drop carriage {:?}",dc.extent().index());
        lock!(self.integration).drop_carriage(dc);
        self.upate_train(dc,-1);
    }

    pub(super) fn set_carriages(&self, extent: &TrainExtent, carriages: &[DrawingCarriage2]) {
        log!("gl/set carriages {:?}",carriages.iter().map(|c| { c.extent().train() }).collect::<Vec<_>>());
        lock!(self.integration).set_carriages(extent,carriages);
    }

    pub(super) fn start_transition(&self, train: &TrainExtent, max: u64, speed: CarriageSpeed) {
        lock!(self.integration).start_transition(train,max,speed);
    }

    pub(super) fn set_playing_field(&self, playing_field: &GlobalPlayingField) {
        let playing_field = PlayingField::new(playing_field);
        lock!(self.integration).set_playing_field(playing_field.clone());
    }

    pub(super) fn set_metadata(&self, metadata: &GlobalAllotmentMetadata) {
        lock!(self.integration).notify_allotment_metadata(metadata);
    }

    pub(super) fn notify_viewport(&self, viewport: &Viewport) {
        lock!(self.integration).notify_viewport(viewport);
    }
}

impl Drop for GraphicsDropper {
    fn drop(&mut self) {
        let trains = lock!(self.trains);
        for train in trains.keys() {
            lock!(self.integration).drop_train(train);
        }
    }
}
