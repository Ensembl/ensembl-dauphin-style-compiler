use anyhow::Context;
use peregrine_core::{ Commander, PeregrineApi, PeregrineObjects, PeregrineIntegration, Carriage, PeregrineApiQueue, ChannelIntegration };
use web_sys::console;
use super::pgcommander::PgCommanderWeb;
use super::pgconsole::PgConsoleWeb;
use super::pgchannel::PgChannel;
use crate::util::error::{ js_option };
use peregrine_dauphin_queue::{ PgDauphinQueue };

fn setup_commander() -> anyhow::Result<PgCommanderWeb> {
    let window = js_option(web_sys::window(),"cannot get window")?;
    let document = js_option(window.document(),"cannot get document")?;
    let html = js_option(document.body().clone(),"cannot get body")?;
    let commander = PgCommanderWeb::new(&html)?;
    commander.start();
    Ok(commander)
}

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

    fn set_carriages(&mut self, carriages: &[Carriage], quick: bool) {
        let carriages : Vec<_> = carriages.iter().map(|x| x.id().to_string()).collect();
        console::log_1(&format!("carriages={:?} quick={:?}",carriages.join(", "),quick).into());
    }

    fn channel(&self) -> Box<dyn ChannelIntegration> {
        Box::new(self.channel.clone())
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

// XXX empty initial layout. 
