use anyhow::{ self, anyhow as err };
use blackbox::blackbox_log;
use dauphin_interp::{ CommandInterpretSuite, Dauphin, InterpretInstance, make_core_interp };
use dauphin_lib_std::make_std_interp;
use commander::cdr_tick;
use serde_cbor::Value as CborValue;
use std::collections::HashMap;
use std::sync::{ Arc, Mutex };
use crate::request::channel::Channel;
use crate::request::packet::ResponsePacket;
use crate::request::program::ProgramLoader;

pub trait PgDauphinIntegration {
    fn add_payloads(&self, dauphin: &mut Dauphin);
}

fn command_suite() -> anyhow::Result<CommandInterpretSuite> {
    let mut cis = CommandInterpretSuite::new();
    cis.register(make_core_interp())?;
    cis.register(make_std_interp())?;
    Ok(cis)
}

pub struct PgDauphinProcess {
    instance: Box<dyn InterpretInstance>
}

impl PgDauphinProcess {
    fn new(dauphin: &Dauphin, binary_name: &str, name: &str) -> anyhow::Result<PgDauphinProcess> {
        Ok(PgDauphinProcess {
            instance: Box::new(dauphin.run_stepwise(binary_name,name)?)
        })
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        loop {
            let out = self.instance.more()?;
            if !out { break; }
            cdr_tick(0).await;
        }
        Ok(())
    }
}

struct PgDauphinData {
    dauphin: Dauphin,
    names: HashMap<(String,String),Option<(String,String)>>,
}

#[derive(Clone)]
pub struct PgDauphin(Arc<Mutex<PgDauphinData>>);

impl PgDauphin {
    pub fn new(integration: Box<dyn PgDauphinIntegration>) -> anyhow::Result<PgDauphin> {
        let mut dauphin = Dauphin::new(command_suite()?);
        integration.add_payloads(&mut dauphin);
        Ok(PgDauphin(Arc::new(Mutex::new(PgDauphinData {
            dauphin,
            names: HashMap::new(),
        }))))
    }

    pub fn add_binary_direct(&self, binary_name: &str, cbor: &CborValue) -> anyhow::Result<()> {
        self.0.lock().unwrap().dauphin.add_binary(binary_name,cbor)
    }

    fn binary_name(&self, channel: &Channel, name_of_bundle: &str) -> String {
        let channel_name = channel.to_string();
        format!("{}-{}-{}",channel_name.len(),channel_name,name_of_bundle)
    }

    pub fn add_binary(&self, channel: &Channel, name_of_bundle: &str, cbor: &CborValue) -> anyhow::Result<()> {
        self.add_binary_direct(&self.binary_name(channel,name_of_bundle),cbor)
    }

    pub fn load(&self, binary_name: &str, name: &str) -> anyhow::Result<PgDauphinProcess> {
        PgDauphinProcess::new(&self.0.lock().unwrap().dauphin, binary_name, name)
    }

    pub fn add_programs(&self, channel: &Channel, response: &ResponsePacket) -> anyhow::Result<()> {
        for bundle in response.programs().iter() {
            blackbox_log!(&format!("channel-{}",self.channel.to_string()),"registered bundle {}",bundle.bundle_name());
            self.add_binary(channel,bundle.bundle_name(),bundle.program())?;
            let data = self.0.lock().unwrap();
            for (in_channel_name,in_bundle_name) in bundle.name_map() {
                blackbox_log!(&format!("channel-{}",self.channel.to_string()),"registered program {}",in_channel_name);
                self.register(channel,in_channel_name,&self.binary_name(channel,bundle.bundle_name()),in_bundle_name);
            }
        }
        Ok(())
    }

    pub fn register(&self, channel: &Channel, name_in_channel: &str, name_of_bundle: &str, name_in_bundle: &str) {
        let binary_name = self.binary_name(channel,name_of_bundle);
        self.0.lock().unwrap().names.insert((channel.to_string(),name_in_channel.to_string()),Some((binary_name,name_in_bundle.to_string())));
    }

    pub fn is_present(&self, channel: &Channel, name_in_channel: &str) -> bool {
        self.0.lock().unwrap().names.get(&(channel.to_string(),name_in_channel.to_string())).and_then(|x| x.as_ref()).is_some()
    }

    pub fn mark_missing(&self, channel: &Channel, name_in_channel: &str) {
        let mut data = self.0.lock().unwrap();
        data.names.insert((channel.channel_name(),name_in_channel.to_string()),None);
    }

    pub async fn load_program(&self, loader: &ProgramLoader, channel: &Channel, program_name: &str) -> anyhow::Result<PgDauphinProcess> {
        if !self.is_present(channel,program_name) {
            loader.load(channel,program_name).await?;
        }
        let data = self.0.lock().unwrap();
        let (bundle_name,in_bundle_name) = data.names.get(&(channel.to_string(),program_name.to_string())).as_ref().unwrap().as_ref()
            .ok_or(err!("Failed channel/program = {}/{}",channel.to_string(),program_name))?.to_owned();
        drop(data);
        self.load(&bundle_name,&in_bundle_name)
    }
}
