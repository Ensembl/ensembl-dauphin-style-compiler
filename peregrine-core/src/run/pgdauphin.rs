use anyhow::{ self, anyhow as err };
use blackbox::blackbox_log;
use dauphin_interp::{ CommandInterpretSuite, Dauphin, InterpretInstance, make_core_interp };
use dauphin_lib_std::make_std_interp;
use commander::{ cdr_tick, RunSlot, CommanderStream };
use serde_cbor::Value as CborValue;
use std::collections::HashMap;
use std::sync::{ Arc, Mutex };
use super::pgcommander::{ PgCommander, PgCommanderTaskSpec };
use crate::request::channel::Channel;
use crate::request::packet::ResponsePacket;
use crate::request::program::ProgramLoader;

pub struct PgDauphinTaskSpec {
    pub prio: i8, 
    pub slot: Option<RunSlot>, 
    pub timeout: Option<f64>,
    pub channel: Channel,
    pub program_name: String
}

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

struct DauphinRunnerRequest {
    task: PgCommanderTaskSpec<()>,
    finishstream: CommanderStream<DauphinResponse>
}

impl DauphinRunnerRequest {
    fn new(task: PgCommanderTaskSpec<()>) -> DauphinRunnerRequest {
        DauphinRunnerRequest {
            task,
            finishstream: CommanderStream::new()
        }
    }

    fn waiter(&self) -> CommanderStream<DauphinResponse> {
        self.finishstream.clone()
    }
}

struct DauphinResponse {

}

impl DauphinResponse {
    fn new() -> DauphinResponse {
        DauphinResponse {

        }
    }
}

async fn run_task(commander: &PgCommander, req: DauphinRunnerRequest) -> anyhow::Result<()> {
    let task = PgCommanderTaskSpec {
        name: req.task.name.to_string(),
        prio: req.task.prio,
        slot: req.task.slot.clone(),
        timeout: req.task.timeout,
        task: Box::pin(async {
            req.task.task.await?;
            req.finishstream.add(DauphinResponse::new());
            Ok(())
        })
    };
    commander.add_task(task);
    Ok(())
}

async fn runner(commander: PgCommander, stream: CommanderStream<DauphinRunnerRequest>) -> anyhow::Result<()> {
    loop {
        let mut request = stream.get_multi().await;
        for r in request.drain(..) {
            run_task(&commander,r).await?;    
        }
    }
}

struct PgDauphinData {
    requests: CommanderStream<DauphinRunnerRequest>,
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
            requests: CommanderStream::new(),
            dauphin,
            names: HashMap::new(),
        }))))
    }

    pub fn start_runner(&self, commander: &PgCommander) {
        commander.add_task(PgCommanderTaskSpec {
            name: "dauphin runner".to_string(),
            prio: 2,
            slot: None,
            timeout: None,
            task: Box::pin(runner(commander.clone(),self.0.lock().unwrap().requests.clone()))
        });
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

    pub async fn run_program(&self, loader: &ProgramLoader, spec: PgDauphinTaskSpec) -> anyhow::Result<()> {
        if !self.is_present(&spec.channel,&spec.program_name) {
            loader.load(&spec.channel,&spec.program_name).await?;
        }
        let data = self.0.lock().unwrap();
        let (bundle_name,in_bundle_name) = data.names.get(&(spec.channel.to_string(),spec.program_name.to_string())).as_ref().unwrap().as_ref()
            .ok_or(err!("Failed channel/program = {}/{}",spec.channel.to_string(),spec.program_name))?.to_owned();
        drop(data);
        let process = self.load(&bundle_name,&in_bundle_name)?;
        let drr = DauphinRunnerRequest::new(PgCommanderTaskSpec {
            name: format!("dauphin: {}",spec.program_name),
            prio: spec.prio,
            slot: spec.slot.clone(),
            timeout: spec.timeout,
            task: Box::pin(process.run())
        });
        let waiter = drr.waiter().clone();
        self.0.lock().unwrap().requests.add(drr);
        waiter.get().await;
        Ok(())
    }
}
