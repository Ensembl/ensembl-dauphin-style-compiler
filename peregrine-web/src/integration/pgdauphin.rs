use anyhow;
use dauphin_interp::{ CommandInterpretSuite, Dauphin, InterpretInstance, make_core_interp };
use dauphin_lib_std::make_std_interp;
use commander::cdr_tick;
use crate::integration::stream::WebStreamFactory;
use serde_cbor::Value as CborValue;

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
    fn new(dauphin: &Dauphin, name: &str) -> anyhow::Result<PgDauphinProcess> {
        Ok(PgDauphinProcess {
            instance: Box::new(dauphin.run_stepwise(name)?)
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
    dauphin: Dauphin
}

impl PgDauphin {
    pub fn new() -> anyhow::Result<PgDauphin> {
        let mut dauphin = Dauphin::new(command_suite()?);
        dauphin.add_payload_factory("std","stream",Box::new(WebStreamFactory::new()));
        Ok(PgDauphin {
            dauphin
        })
    }

    pub fn add_binary(&mut self, cbor: &CborValue) -> anyhow::Result<()> {
        self.dauphin.add_binary(cbor)
    }

    pub fn load(&self, name: &str) -> anyhow::Result<PgDauphinProcess> {
        PgDauphinProcess::new(&self.dauphin,name)
    }
}
