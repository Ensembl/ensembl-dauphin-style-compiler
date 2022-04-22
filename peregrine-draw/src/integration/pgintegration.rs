use std::sync::{ Arc, Mutex };
use peregrine_data::{
    Assets, CarriageSpeed, ChannelIntegration, PeregrineIntegration, 
    Viewport, TrainExtent, DrawingCarriage2, GlobalAllotmentMetadata, PlayingField
};
use peregrine_toolkit::{lock, log};
use super::pgchannel::PgChannel;
use crate::{PeregrineDom};
use crate::input::Input;
use crate::run::report::Report;
use crate::train::GlRailway;
use peregrine_data::{ DataMessage };
use crate::webgl::global::WebGlGlobal;
use crate::stage::stage::Stage;
pub struct PgIntegration {
    channel: PgChannel,
    trainset: GlRailway,
    input: Input,
    webgl: Arc<Mutex<WebGlGlobal>>,
    stage: Arc<Mutex<Stage>>,
    report: Report,
    dom: PeregrineDom,
    assets: Assets
}

impl PeregrineIntegration for PgIntegration {
    fn set_assets(&mut self, mut assets: Assets) {
        self.assets.add(&mut assets);
    }

    fn create_train(&mut self, train: &TrainExtent) {
        self.trainset.create_train(train);
    }

    fn drop_train(&mut self, train: &TrainExtent) {
        self.trainset.drop_train(train);
    }

    fn create_carriage(&mut self, carriage: &DrawingCarriage2) {
        self.trainset.create_carriage(carriage,&self.webgl,&self.assets);
    }

    fn drop_carriage(&mut self, carriage: &DrawingCarriage2) {
        self.trainset.drop_carriage(carriage);
    }

    fn set_carriages(&mut self, train: &TrainExtent, carriages: &[DrawingCarriage2]) -> Result<(),DataMessage> {
        self.trainset.set_carriages(train,carriages);
        Ok(())
    }

    fn notify_allotment_metadata(&mut self, metadata: &GlobalAllotmentMetadata) {
        self.report.set_allotter_metadata(metadata);
    }

    fn channel(&self) -> Box<dyn ChannelIntegration> {
        Box::new(self.channel.clone())
    }

    fn start_transition(&mut self, extent: &TrainExtent, max: u64, speed: CarriageSpeed) ->Result<(),DataMessage> {
        self.input.set_limit(max as f64);
        self.trainset.start_fade(extent,max,speed)
            .map_err(|e| DataMessage::TunnelError(Arc::new(Mutex::new(e))))?;
        Ok(())
    }

    fn notify_viewport(&mut self, viewport: &Viewport) {
        self.stage.lock().unwrap().notify_current(viewport);
        if let Ok(layout) = viewport.layout() {
            let stick = layout.stick();
            self.report.set_stick(&stick.to_string());
            if let (Ok(x),Ok(bp)) = (viewport.position(),viewport.bp_per_screen()) {
                self.report.set_x_bp(x);
                self.report.set_bp_per_screen(bp);
            }
        }
    }

    fn set_playing_field(&mut self, playing_field: PlayingField) {
        self.dom.set_useful_height(playing_field.height as u32);
        lock!(self.stage).notify_playingfield(&playing_field);
    }
}

impl PgIntegration {
    pub(crate) fn new(channel: PgChannel, trainset: GlRailway, input: &Input, webgl: Arc<Mutex<WebGlGlobal>>, stage: &Arc<Mutex<Stage>>, dom: &PeregrineDom, report: &Report) -> PgIntegration {
        PgIntegration {
            channel,
            trainset,
            webgl,
            stage: stage.clone(),
            report: report.clone(),
            dom: dom.clone(),
            input: input.clone(),
            assets: Assets::empty()
        }
    }

    pub(crate) fn assets(&self) -> &Assets { &self.assets }
}
