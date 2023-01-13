use commander::{ CommanderStream, cdr_tick };
use eard_interp::{LibcoreTemplate, InterpreterBuilder, build_libcore, Interpreter, LibcoreBuilder, RunContext, prepare_libcore};
use eard_libeoe::{ build_libeoe, LibEoEBuilder, prepare_libeoe };
use peregrine_data::{ 
    PgCommander, PgCommanderTaskSpec, InstancePayload, PeregrineCore, add_task, DataStore
};
use peregrine_dauphin_queue::{ PgDauphinTaskSpec, PgDauphinRunTaskSpec, PgDauphinLoadTaskSpec, PgEardoLoadTaskSpec, PgEardoRunTaskSpec };
use dauphin_interp::{ Dauphin, CommandInterpretSuite, InterpretInstance, make_core_interp, PayloadFactory };
use dauphin_lib_std::make_std_interp;
use dauphin_lib_peregrine::{ make_peregrine_interp, add_peregrine_payloads };
use eard_libperegrine::{build_libperegrine, prepare_libperegrine, LibPeregrineBuilder};
use peregrine_toolkit::error::Error;
use peregrine_toolkit::{log_extra, log};
use std::any::Any;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

pub trait PgDauphinIntegration {
    fn add_payloads(&self, dauphin: &mut Dauphin);
}

pub struct Process {
    instance: Box<dyn InterpretInstance>
}

impl Process {
    fn new(dauphin: &Dauphin, name: &str, instance: HashMap<String,Box<dyn Any>>) -> anyhow::Result<Process> {
        let instance = InstancePayload::new(instance);
        let mut more_payloads = HashMap::new();
        more_payloads.insert(("peregrine".to_string(),"instance".to_string()),Box::new(instance) as Box<dyn PayloadFactory>);
        Ok(Process {
            instance: Box::new(dauphin.run_stepwise(name,&more_payloads)?)
        })
    }

    pub async fn run(mut self) -> Result<(),Error> {
        loop {
            let out = self.instance.more().await.map_err(|e| Error::operr(&format!("XXXTmp wrap {:?}",e)))?;
            if !out { break; }
            cdr_tick(0).await;
        }
        Ok(())
    }
}

fn command_suite() -> anyhow::Result<CommandInterpretSuite> {
    let mut cis = CommandInterpretSuite::new();
    cis.register(make_core_interp())?;
    cis.register(make_std_interp())?;
    cis.register(make_peregrine_interp())?;
    Ok(cis)
}

fn load(dauphin: &mut Dauphin, spec: PgDauphinLoadTaskSpec, stream: CommanderStream<Result<(),Error>>) {
    stream.add(dauphin.add_binary(&spec.data));
}

fn load_eardo(interp: &mut Interpreter, spec: PgEardoLoadTaskSpec, stream: CommanderStream<Result<(),Error>>) {
    stream.add(interp.load(&spec.data).map_err(|e|
        Error::operr(&format!("Cannot load eardo: {}",e))
    ));
}

fn run(dauphin: &mut Dauphin, commander: &PgCommander, spec: PgDauphinRunTaskSpec, stream: CommanderStream<Result<(),Error>>) {
    match Process::new(dauphin,&spec.in_bundle_name,spec.payloads) {
        Ok(process) => {
            let stream = stream.clone();
            let task = PgCommanderTaskSpec {
                name: format!("dauphin: {} {}",spec.bundle_name,spec.in_bundle_name),
                prio: spec.prio,
                slot: spec.slot.clone(),
                timeout: spec.timeout,
                task: Box::pin(async move {
                    stream.add(process.run().await);
                    Ok(())
                }),
                stats: true
            };
            add_task(&commander,task);
        },
        Err(e) => {
            stream.add(Err(Error::operr(&format!("XXXTmp wrap {:?}",e))));
        }
    }
}

macro_rules! result {
    ($value:expr, $stream:expr, $out:expr) => {
        match $value {
            Ok(x) => x,
            Err(e) => {
                $stream.add(Err(Error::operr(&format!("running eardo: {}",e))));
                return $out;
            }
        }
    };
}

fn run_eardo(interp: &mut Interpreter, data_store: &DataStore, 
    libcore_builder: &LibcoreBuilder, libperegrine_builder: &LibPeregrineBuilder, libeoe_builder: &LibEoEBuilder,
    commander: &PgCommander, spec: PgEardoRunTaskSpec, stream: CommanderStream<Result<(),Error>>) {
    /* run */
    let stream = stream.clone();
    let program = result!(interp.get(&spec.name,"main"),stream,()).clone();
    let libcore_builder = libcore_builder.clone();
    let libperegrine_builder = libperegrine_builder.clone();
    let libeoe_builder = libeoe_builder.clone();
    let data_store = data_store.clone();
    let task = PgCommanderTaskSpec {
        name: format!("eard: {:?}",spec.name),
        prio: spec.prio,
        slot: None,
        timeout: None,
        task: Box::pin(async move {
            let mut context = RunContext::new();
            prepare_libcore(&mut context,&libcore_builder,LibcoreBrowser);
            result!(prepare_libperegrine(
                &mut context,&libperegrine_builder,&data_store,
                spec.payloads
            ),stream,Ok(()));
            result!(prepare_libeoe(&mut context,&libeoe_builder),stream,Ok(()));
            result!(program.run(context).await,stream,Ok(()));
            stream.add(Ok(()));
            Ok(())
        }),
        stats: true
    };
    add_task(&commander,task);
}

async fn call_up_async() -> Result<(),String> {
    Ok(())
}

struct LibcoreBrowser;

impl LibcoreTemplate for LibcoreBrowser {
    fn print(&self, s: &str) {
        log!("{}",s);
    }

    fn call_up(&self) -> Pin<Box<dyn Future<Output=Result<(),String>>>> {
        Box::pin(call_up_async())
    }    
}

fn eard_interp() -> Result<(Interpreter,LibcoreBuilder,LibPeregrineBuilder,LibEoEBuilder),String> {
    let mut builder = InterpreterBuilder::new();
    builder.set_step_by_step(true);
    let libcore_builder = build_libcore(&mut builder)?;
    let libperegrine_builder = build_libperegrine(&mut builder)?;
    let libeoe_builder = build_libeoe(&mut builder)?;
    Ok((Interpreter::new(builder),libcore_builder,libperegrine_builder,libeoe_builder))
}

async fn main_loop(integration: Box<dyn PgDauphinIntegration>, core: PeregrineCore) -> Result<(),Error> {
    let mut dauphin = Dauphin::new(command_suite().map_err(|e| Error::fatal(&format!("cannot run style compiler {}",e.to_string())))?);
    integration.add_payloads(&mut dauphin);
    add_peregrine_payloads(&mut dauphin,&core.agent_store);
    let data_store = core.agent_store.data_store.clone();
    let (mut interp,libcore_builder,libperegrine_builder,libeoe_builder) = eard_interp().map_err(|e| Error::operr(&e))?;
    loop {
        let e = core.base.dauphin_queue.get().await;
        match e.task {
            PgDauphinTaskSpec::Load(p) => load(&mut dauphin,p,e.channel),
            PgDauphinTaskSpec::LoadEardo(p) => load_eardo(&mut interp,p,e.channel),
            PgDauphinTaskSpec::Run(r) => run(&mut dauphin,&core.base.commander,r,e.channel),
            PgDauphinTaskSpec::RunEardo(r) => run_eardo(&mut interp,&data_store,&libcore_builder,&libperegrine_builder,&libeoe_builder,&core.base.commander,r,e.channel),
            PgDauphinTaskSpec::Quit => { break; }
        }
    }
    log_extra!("dauphin runner quit");
    Ok(())
}

pub fn peregrine_dauphin(integration: Box<dyn PgDauphinIntegration>, core: &PeregrineCore) {
    add_task(&core.base.commander,PgCommanderTaskSpec {
        name: "dauphin runner".to_string(),
        prio: 2,
        slot: None,
        timeout: None,
        task: Box::pin(main_loop(integration,core.clone())),
        stats: false
    });
}
