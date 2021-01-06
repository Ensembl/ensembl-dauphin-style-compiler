use anyhow::Context;
use peregrine_core::{ Commander, PeregrineApi, CarriageSpeed, PeregrineObjects, PeregrineIntegration, Carriage, PeregrineApiQueue, ChannelIntegration };
use web_sys::console;
use super::pgchannel::PgChannel;
use crate::util::error::{ js_option, js_throw };
use blackbox::blackbox_log;
use crate::train::GlTrainSet;
use peregrine_core::PeregrineConfig;

pub struct PgIntegration {
    channel: PgChannel,
    trainset: GlTrainSet
}

impl PeregrineIntegration for PgIntegration {
    fn set_api(&mut self, api: PeregrineApi) {
    }

    fn report_error(&mut self, error: &str) {
        console::warn_1(&format!("{}\n",error).into());
    }

    fn set_carriages(&mut self, carriages: &[Carriage], index: u32) {
        let carriages_str : Vec<_> = carriages.iter().map(|x| x.id().to_string()).collect();
        blackbox_log!("uiapi","set_carriages(carriages={:?}({}) index={:?})",carriages_str.join(", "),carriages_str.len(),index);
        self.trainset.set_carriages(carriages,index);
    }

    fn channel(&self) -> Box<dyn ChannelIntegration> {
        Box::new(self.channel.clone())
    }

    fn start_transition(&mut self, index: u32, max: u64, speed: CarriageSpeed) {
        blackbox_log!("uiapi","start_transition(index={} max={} speed={:?})",index,max,speed);
        js_throw(self.trainset.start_fade(index,max,speed));
    }
}

impl PgIntegration {
    pub fn new(channel: PgChannel, trainset: GlTrainSet) -> PgIntegration {
        PgIntegration {
            channel,
            trainset
        }
    }
}
