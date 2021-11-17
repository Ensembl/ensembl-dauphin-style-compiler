use std::sync::{ Arc, Mutex };
use peregrine_data::{
    AllotmentMetadataReport, Assets, Carriage, CarriageSpeed, ChannelIntegration, PeregrineIntegration, PlayingField, 
    Scale, Viewport
};
use peregrine_toolkit::lock;
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

    fn set_carriages(&mut self, carriages: &[Carriage], scale: Scale, index: u32) -> Result<(),DataMessage> {
        let mut webgl = self.webgl.lock().unwrap();
        self.trainset.set_carriages(carriages,&scale,&mut webgl,&self.assets,index)
            .map_err(|e| DataMessage::TunnelError(Arc::new(Mutex::new(e))))?;
        Ok(())
    }

    fn notify_allotment_metadata(&mut self, metadata: &AllotmentMetadataReport) {
        self.report.set_allotter_metadata(metadata);
    }

    fn channel(&self) -> Box<dyn ChannelIntegration> {
        Box::new(self.channel.clone())
    }

    fn start_transition(&mut self, index: u32, max: u64, speed: CarriageSpeed) ->Result<(),DataMessage> {
        self.input.set_limit(max as f64);
        self.trainset.start_fade(index,max,speed)
            .map_err(|e| DataMessage::TunnelError(Arc::new(Mutex::new(e))))?;
        Ok(())
    }

    fn notify_viewport(&mut self, viewport: &Viewport, future: bool) {
        if !future {
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
    }

    fn set_playing_field(&mut self, playing_field: PlayingField) {
        self.dom.set_useful_height(playing_field.height() as u32);
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
