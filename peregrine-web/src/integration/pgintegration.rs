use anyhow::Context;
use peregrine_core::{ Commander, PeregrineApi, CarriageSpeed, PeregrineObjects, PeregrineIntegration, Carriage, PeregrineApiQueue, ChannelIntegration };
use web_sys::console;
use super::pgchannel::PgChannel;
use crate::util::error::{ js_option };
use blackbox::blackbox_log;

pub struct PgIntegration {
    api: Option<PeregrineApi>,
    channel: PgChannel
}

impl PeregrineIntegration for PgIntegration {
    fn set_api(&mut self, api: PeregrineApi) {
        self.api = Some(api);
    }

    fn report_error(&mut self, error: &str) {
        console::warn_1(&format!("{}\n",error).into());
    }

    fn set_carriages(&mut self, carriages: &[Carriage], index: u32) {
        let carriages : Vec<_> = carriages.iter().map(|x| x.id().to_string()).collect();
        blackbox_log!("uiapi","set_carriages(carriages={:?}({}) index={:?})",carriages.join(", "),carriages.len(),index);
    }

    fn channel(&self) -> Box<dyn ChannelIntegration> {
        Box::new(self.channel.clone())
    }

    fn start_transition(&mut self, index: u32, max: u64, speed: CarriageSpeed) {
        blackbox_log!("uiapi","start_transition(index={} max={} speed={:?}",index,max,speed);
        if let Some(api) = &self.api {
            api.transition_complete();
        }
    }
}

impl PgIntegration {
    //        let console = PgConsoleWeb::new(30,30.);
    //let channel = PgChannel::new(Box::new(console.clone()));
    pub fn new(channel: PgChannel) -> PgIntegration {
        PgIntegration {
            api: None,
            channel
        }
    }
}
