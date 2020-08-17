use anyhow::{ self, anyhow as err };
use dauphin_interp::{ CommandInterpretSuite, Dauphin, InterpretInstance, make_core_interp };
use dauphin_lib_std::make_std_interp;
use commander::cdr_tick;
use serde_cbor::Value as CborValue;
use std::collections::HashMap;
use crate::request::request::ResponsePacket;

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

pub struct PgDauphin {
    dauphin: Dauphin,
    names: HashMap<(String,String),(String,String)>
}

impl PgDauphin {
    pub fn new(integration: Box<dyn PgDauphinIntegration>) -> anyhow::Result<PgDauphin> {
        let mut dauphin = Dauphin::new(command_suite()?);
        integration.add_payloads(&mut dauphin);
        Ok(PgDauphin {
            dauphin,
            names: HashMap::new()
        })
    }

    pub fn add_binary(&mut self, binary_name: &str, cbor: &CborValue) -> anyhow::Result<()> {
        self.dauphin.add_binary(binary_name,cbor)
    }

    pub fn load(&self, binary_name: &str, name: &str) -> anyhow::Result<PgDauphinProcess> {
        PgDauphinProcess::new(&self.dauphin, binary_name, name)
    }

    pub fn add_programs(&mut self, response: &ResponsePacket) -> anyhow::Result<()> {
        let channel = response.channel_identity();
        for bundle in response.programs().iter() {
            let program_name = format!("{}-{}-{}",channel.len(),channel,bundle.bundle_name());
            self.add_binary(&program_name,bundle.program())?;
            for (in_channel_name,in_bundle_name) in bundle.name_map() {
                self.names.insert((channel.to_string(),in_channel_name.to_string()),
                                  (program_name.clone(),in_bundle_name.to_string()));
            }
        }
        Ok(())
    }

    pub fn load_program(&self, channel_name: &str, program_name: &str) -> anyhow::Result<PgDauphinProcess> {
        let (bundle_name,in_bundle_name) = self.names.get(&(channel_name.to_string(),program_name.to_string()))
            .ok_or(err!("No such channel/program = {}/{}",channel_name,program_name))?;
        self.load(bundle_name,in_bundle_name)
    }
}
