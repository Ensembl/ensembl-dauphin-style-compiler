use anyhow::{ self, anyhow as err };
use dauphin_interp::{ CommandInterpretSuite, Dauphin, InterpretInstance, make_core_interp };
use dauphin_lib_std::make_std_interp;
use commander::cdr_tick;
use serde_cbor::Value as CborValue;
use std::collections::HashMap;
use std::sync::{ Arc, Mutex };
use crate::request::channel::Channel;
use crate::request::manager::RequestManager;
use crate::request::packet::ResponsePacket;
use crate::request::program::ProgramLoader;
use crate::run::PgCommander;

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

    pub fn add_binary(&self, binary_name: &str, cbor: &CborValue) -> anyhow::Result<()> {
        self.0.lock().unwrap().dauphin.add_binary(binary_name,cbor)
    }

    pub fn load(&self, binary_name: &str, name: &str) -> anyhow::Result<PgDauphinProcess> {
        PgDauphinProcess::new(&self.0.lock().unwrap().dauphin, binary_name, name)
    }

    pub fn add_programs(&self, response: &ResponsePacket) -> anyhow::Result<()> {
        let channel = response.channel_identity();
        for bundle in response.programs().iter() {
            let program_name = format!("{}-{}-{}",channel.len(),channel,bundle.bundle_name());
            self.add_binary(&program_name,bundle.program())?;
            let mut data = self.0.lock().unwrap();
            for (in_channel_name,in_bundle_name) in bundle.name_map() {
                data.names.insert((channel.to_string(),in_channel_name.to_string()),
                                  Some((program_name.clone(),in_bundle_name.to_string())));
            }
        }
        Ok(())
    }

    pub fn mark_missing(&self, channel: &Channel, program_name: &str) {
        let mut data = self.0.lock().unwrap();
        data.names.insert((channel.channel_name(),program_name.to_string()),None);
    }

    pub async fn load_program(&self, loader: &ProgramLoader, channel: &Channel, program_name: &str) -> anyhow::Result<PgDauphinProcess> {
        let channel_name = channel.channel_name();
        let key = (channel_name.to_string(),program_name.to_string());
        let data = self.0.lock().unwrap();
        let missing = !data.names.contains_key(&key);
        drop(data);
        if missing {
            loader.load(channel,program_name).await?;
        }
        let data = self.0.lock().unwrap();
        let (bundle_name,in_bundle_name) = data.names.get(&key).as_ref().unwrap().as_ref()
            .ok_or(err!("Failed channel/program = {}/{}",channel_name,program_name))?;
        self.load(&bundle_name,&in_bundle_name)
    }
}
