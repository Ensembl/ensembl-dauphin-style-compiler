use std::sync::{ Arc, Mutex };
use commander::{FusePromise, PromiseFuture};
use peregrine_data::{Carriage, CarriageSpeed, ChannelIntegration, PeregrineIntegration, StickId, Viewport};
use super::busywaiter::BusyWaiter;
use super::pgchannel::PgChannel;
use crate::input::Input;
use crate::run::report::Report;
use crate::train::GlTrainSet;
use peregrine_data::{ DataMessage };
use crate::webgl::global::WebGlGlobal;
use crate::stage::stage::Stage;

pub struct PgIntegration {
    busy_waiter: BusyWaiter,
    channel: PgChannel,
    trainset: GlTrainSet,
    webgl: Arc<Mutex<WebGlGlobal>>,
    stage: Arc<Mutex<Stage>>,
    input: Input,
    report: Report
}

impl PeregrineIntegration for PgIntegration {
    fn set_carriages(&mut self, carriages: &[Carriage], index: u32) -> Result<(),DataMessage> {
        let mut webgl = self.webgl.lock().unwrap();
        self.trainset.set_carriages(carriages,&mut webgl,index)
            .map_err(|e| DataMessage::TunnelError(Arc::new(Mutex::new(e))))?;
        Ok(())
    }

    fn channel(&self) -> Box<dyn ChannelIntegration> {
        Box::new(self.channel.clone())
    }

    fn start_transition(&mut self, index: u32, max: u64, speed: CarriageSpeed) ->Result<(),DataMessage> {
        let webgl = self.webgl.lock().unwrap();
        self.trainset.start_fade(&webgl,index,max,speed)
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

    fn busy(&mut self, yn: bool) {
        self.busy_waiter.set(yn);
    }
}

impl PgIntegration {
    pub(crate) fn new(channel: PgChannel, input: &Input, trainset: GlTrainSet, webgl: Arc<Mutex<WebGlGlobal>>, stage: &Arc<Mutex<Stage>>, report: &Report, busy_waiter: &BusyWaiter) -> PgIntegration {
        PgIntegration {
            busy_waiter: busy_waiter.clone(),
            channel,
            trainset,
            webgl,
            stage: stage.clone(),
            input: input.clone(),
            report: report.clone()
        }
    }
}
