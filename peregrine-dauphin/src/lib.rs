use blackbox::blackbox_log;
use commander::{ CommanderStream, cdr_tick };
use peregrine_data::{ 
    PgCommander, PgCommanderTaskSpec, StickAuthorityStore, InstancePayload, RequestManager, StickStore, CountingPromise, PanelProgramStore,
    DataStore, PeregrineCore
};
use peregrine_dauphin_queue::{ PgDauphinQueue, PgDauphinTaskSpec, PgDauphinRunTaskSpec, PgDauphinLoadTaskSpec };
use dauphin_interp::{ Dauphin, CommandInterpretSuite, InterpretInstance, make_core_interp, PayloadFactory, Payload };
use dauphin_lib_std::make_std_interp;
use dauphin_lib_peregrine::{ make_peregrine_interp, add_peregrine_payloads };
use std::any::Any;
use std::collections::HashMap;

pub trait PgDauphinIntegration {
    fn add_payloads(&self, dauphin: &mut Dauphin);
}

pub struct Process {
    instance: Box<dyn InterpretInstance>
}

impl Process {
    fn new(dauphin: &Dauphin, binary_name: &str, name: &str, instance: HashMap<String,Box<dyn Any>>) -> anyhow::Result<Process> {
        let instance = InstancePayload::new(instance);
        let mut more_payloads = HashMap::new();
        more_payloads.insert(("peregrine".to_string(),"instance".to_string()),Box::new(instance) as Box<dyn PayloadFactory>);
        blackbox_log!("dauphin","process {} {}",binary_name,name);
        Ok(Process {
            instance: Box::new(dauphin.run_stepwise(binary_name,name,&more_payloads)?)
        })
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        loop {
            let out = self.instance.more().await?;
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

fn load(dauphin: &mut Dauphin, spec: PgDauphinLoadTaskSpec, stream: CommanderStream<anyhow::Result<()>>) {
    blackbox_log!("dauphin","load {}",spec.bundle_name);
    stream.add(dauphin.add_binary(&spec.bundle_name,&spec.data));
}

fn run(dauphin: &mut Dauphin, commander: &PgCommander, spec: PgDauphinRunTaskSpec, stream: CommanderStream<anyhow::Result<()>>) {
    blackbox_log!("dauphin","run {} {}",spec.bundle_name,spec.in_bundle_name);
    match Process::new(dauphin,&spec.bundle_name,&spec.in_bundle_name,spec.payloads) {
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
                })
            };
            commander.add_task(task);        
        },
        Err(e) => {
            blackbox_log!("dauphin","{} failed {}",spec.in_bundle_name,e.to_string());
            stream.add(Err(e));
        }
    }
}

async fn main_loop(integration: Box<dyn PgDauphinIntegration>, core: PeregrineCore) -> anyhow::Result<()> {
    let mut dauphin = Dauphin::new(command_suite()?);
    integration.add_payloads(&mut dauphin);
    add_peregrine_payloads(&mut dauphin,&core.manager,&core.stick_store,&core.stick_authority_store,&core.booted,&core.panel_program_store,&core.data_store);
    loop {
        let e = core.dauphin_queue.get().await;
        match e.task {
            PgDauphinTaskSpec::Load(p) => load(&mut dauphin,p,e.channel),
            PgDauphinTaskSpec::Run(r) => run(&mut dauphin,&core.commander,r,e.channel)
        }
    }
}

pub fn peregrine_dauphin(integration: Box<dyn PgDauphinIntegration>, core: &PeregrineCore) {
    core.commander.add_task(PgCommanderTaskSpec {
        name: "dauphin runner".to_string(),
        prio: 2,
        slot: None,
        timeout: None,
        task: Box::pin(main_loop(integration,core.clone()))
    });
}
