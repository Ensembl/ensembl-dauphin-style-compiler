use commander::{ CommanderStream, cdr_tick };
use peregrine_data::{ 
    PgCommander, PgCommanderTaskSpec, InstancePayload, PeregrineCore, add_task
};
use peregrine_dauphin_queue::{ PgDauphinTaskSpec, PgDauphinRunTaskSpec, PgDauphinLoadTaskSpec };
use dauphin_interp::{ Dauphin, CommandInterpretSuite, InterpretInstance, make_core_interp, PayloadFactory };
use dauphin_lib_std::make_std_interp;
use dauphin_lib_peregrine::{ make_peregrine_interp, add_peregrine_payloads };
use peregrine_toolkit::error::Error;
use peregrine_toolkit::log_extra;
use std::any::Any;
use std::collections::HashMap;

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

async fn main_loop(integration: Box<dyn PgDauphinIntegration>, core: PeregrineCore) -> Result<(),Error> {
    let mut dauphin = Dauphin::new(command_suite().map_err(|e| Error::fatal(&format!("cannot run style compiler {}",e.to_string())))?);
    integration.add_payloads(&mut dauphin);
    add_peregrine_payloads(&mut dauphin,&core.agent_store);
    loop {
        let e = core.base.dauphin_queue.get().await;
        match e.task {
            PgDauphinTaskSpec::Load(p) => load(&mut dauphin,p,e.channel),
            PgDauphinTaskSpec::Run(r) => run(&mut dauphin,&core.base.commander,r,e.channel),
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
