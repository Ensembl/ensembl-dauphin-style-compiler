use std::sync::{ Arc, Mutex };
use peregrine_data::{
    Assets, CarriageSpeed, PeregrineIntegration, 
    Viewport, DrawingCarriage, GlobalAllotmentMetadata, PlayingField, TrainIdentity,
    InstanceInformation
};
use peregrine_toolkit::{lock, log};
use crate::domcss::dom::PeregrineDom;
use crate::input::Input;
use crate::run::report::Report;
use crate::train::GlRailway;
use peregrine_data::{ DataMessage };
use crate::webgl::global::WebGlGlobal;
use crate::stage::stage::Stage;

pub struct PgIntegration {
    trainset: GlRailway,
    input: Input,
    webgl: Arc<Mutex<WebGlGlobal>>,
    stage: Arc<Mutex<Stage>>,
    report: Report,
    dom: PeregrineDom,
    assets: Assets,
    paused: bool
}

impl PeregrineIntegration for PgIntegration {
    fn set_assets(&mut self, assets: &Assets) {
        self.assets.add(&assets);
    }

    fn set_pause(&mut self, yn: bool) {
        self.paused = yn;
    }

    fn create_train(&mut self, train: &TrainIdentity) {
        self.trainset.create_train(train);
    }

    fn drop_train(&mut self, train: &TrainIdentity) {
        self.trainset.drop_train(train);
    }

    fn create_carriage(&mut self, carriage: &DrawingCarriage) {
        self.trainset.create_carriage(carriage,&self.webgl,&self.assets);
    }

    fn drop_carriage(&mut self, carriage: &DrawingCarriage) {
        self.trainset.drop_carriage(carriage);
    }

    fn set_carriages(&mut self, train: &TrainIdentity, carriages: &[DrawingCarriage]) -> Result<(),DataMessage> {
        self.trainset.set_carriages(train,carriages);
        Ok(())
    }

    fn notify_allotment_metadata(&mut self, metadata: &GlobalAllotmentMetadata) {
        self.report.set_allotter_metadata(metadata);
    }

    fn start_transition(&mut self, extent: &TrainIdentity, max: u64, speed: CarriageSpeed) -> Result<(),DataMessage> {
        self.input.set_limit(max as f64);
        self.trainset.start_fade(extent,max,speed)
            .map_err(|e| DataMessage::XXXTransitional(e))?;
        Ok(())
    }

    fn notify_viewport(&mut self, viewport: &Viewport) {
        if !self.paused {
            self.stage.lock().unwrap().notify_current(viewport);
        }
        if let Ok(layout) = viewport.layout() {
            let stick = layout.stick();
            self.report.set_stick(&stick.get_id().to_string());
            if let (Ok(x),Ok(bp)) = (viewport.position(),viewport.bp_per_screen()) {
                self.report.set_x_bp(x);
                self.report.set_bp_per_screen(bp);
            }
        }
    }

    fn set_playing_field(&mut self, playing_field: PlayingField) {
        self.dom.set_content_height(playing_field.height as u32);
        lock!(self.stage).notify_playingfield(&playing_field);
    }

    fn report_instance_information(&self, info: &InstanceInformation) {        
        log!("{}",info);
    }
}

impl PgIntegration {
    pub(crate) fn new(trainset: GlRailway, input: &Input, webgl: Arc<Mutex<WebGlGlobal>>, stage: &Arc<Mutex<Stage>>, dom: &PeregrineDom, report: &Report) -> PgIntegration {
        PgIntegration {
            trainset,
            webgl,
            stage: stage.clone(),
            report: report.clone(),
            dom: dom.clone(),
            input: input.clone(),
            assets: Assets::empty(),
            paused: false
        }
    }

    pub(crate) fn assets(&self) -> &Assets { &self.assets }
}
