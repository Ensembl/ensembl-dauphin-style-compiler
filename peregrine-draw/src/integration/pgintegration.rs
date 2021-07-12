use std::sync::{ Arc, Mutex };
use peregrine_data::{ CarriageSpeed, PeregrineIntegration, Carriage, ChannelIntegration, Viewport };
use super::pgchannel::PgChannel;
use blackbox::blackbox_log;
use crate::{run::inner::TargetManager, train::GlTrainSet};
use peregrine_data::{ DataMessage };
use crate::webgl::global::WebGlGlobal;
use crate::stage::stage::Stage;

pub struct PgIntegration {
    channel: PgChannel,
    trainset: GlTrainSet,
    webgl: Arc<Mutex<WebGlGlobal>>,
    stage: Arc<Mutex<Stage>>,
    target_manager: Arc<Mutex<TargetManager>>
}

impl PeregrineIntegration for PgIntegration {
    fn set_carriages(&mut self, carriages: &[Carriage], index: u32) -> Result<(),DataMessage> {
        #[cfg(blackbox)]
        let carriages_str : Vec<_> = carriages.iter().map(|x| x.id().to_string()).collect();
        blackbox_log!("uiapi","set_carriages(carriages={:?}({}) index={:?})",carriages_str.join(", "),carriages_str.len(),index);
        let mut webgl = self.webgl.lock().unwrap();
        self.trainset.set_carriages(carriages,&mut webgl,index)
            .map_err(|e| DataMessage::TunnelError(Arc::new(Mutex::new(e))))?;
        Ok(())
    }

    fn channel(&self) -> Box<dyn ChannelIntegration> {
        Box::new(self.channel.clone())
    }

    fn start_transition(&mut self, index: u32, max: u64, speed: CarriageSpeed) ->Result<(),DataMessage> {
        blackbox_log!("uiapi","start_transition(index={} max={} speed={:?})",index,max,speed);
        let webgl = self.webgl.lock().unwrap();
        self.trainset.start_fade(&webgl,index,max,speed)
            .map_err(|e| DataMessage::TunnelError(Arc::new(Mutex::new(e))))?;
        Ok(())
    }

    fn notify_viewport(&mut self, viewport: &Viewport, future: bool) {
        if !future {
            self.stage.lock().unwrap().notify_current(viewport);
        } else {
            self.target_manager.lock().unwrap().update_viewport(viewport);
        }
    }
}

impl PgIntegration {
    pub fn new(channel: PgChannel, trainset: GlTrainSet, webgl: Arc<Mutex<WebGlGlobal>>, stage: &Arc<Mutex<Stage>>, target_manager: &Arc<Mutex<TargetManager>>) -> PgIntegration {
        PgIntegration {
            channel,
            trainset,
            webgl,
            stage: stage.clone(),
            target_manager: target_manager.clone()
        }
    }
}
