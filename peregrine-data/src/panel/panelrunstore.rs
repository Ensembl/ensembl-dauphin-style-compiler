use std::any::Any;
use std::collections::HashMap;
use std::sync::{ Arc };
use crate::agent::agent::Agent;
use crate::request::channel::{ Channel, PacketPriority, ChannelIntegration };
use crate::request::manager::{ RequestManager, PayloadReceiver };
use crate::ProgramLoader;
use crate::run::{ PgCommander, PgDauphin, PgCommanderTaskSpec, PgDauphinTaskSpec, add_task, async_complete_task };
use crate::shape::ShapeOutput;
use crate::util::memoized::{ Memoized, MemoizedType };
use crate::CountingPromise;
use super::panel::Panel;
use super::programdata::ProgramData;
use crate::index::StickStore;
use super::panelprogramstore::PanelProgramStore;
pub use crate::util::message::DataMessage;
use crate::api::{ MessageSender, PeregrineCoreBase, AgentStore };

#[derive(Clone,Debug,Eq,Hash,PartialEq)]
pub struct PanelRun {
    channel: Channel,
    program: String,
    panel: Panel
}

impl PanelRun {
    pub fn new(channel: Channel, program: &str, panel: &Panel) -> PanelRun {
        PanelRun {
            channel: channel.clone(),
            program: program.to_string(),
            panel: panel.clone()
        }
    }
}

#[derive(Clone)]
pub struct PanelRunOutput {
    shapes: ShapeOutput
}

impl PanelRunOutput {
    fn new() -> PanelRunOutput {
        PanelRunOutput {
            shapes: ShapeOutput::new()
        }
    }

    pub fn shapes(&self) -> &ShapeOutput { &self.shapes }
}

async fn run(base: PeregrineCoreBase, agent_store: AgentStore, panel_run: PanelRun) -> Result<Arc<PanelRunOutput>,DataMessage> {
    base.booted.wait().await;
    let mut payloads = HashMap::new();
    let pro = PanelRunOutput::new();
    payloads.insert("panel".to_string(),Box::new(panel_run.panel.clone()) as Box<dyn Any>);
    payloads.insert("out".to_string(),Box::new(pro.clone()) as Box<dyn Any>);
    payloads.insert("data".to_string(),Box::new(ProgramData::new()) as Box<dyn Any>);
    base.dauphin.run_program(&agent_store.program_loader().await,PgDauphinTaskSpec {
        prio: 1,
        slot: None,
        timeout: None,
        channel: panel_run.channel.clone(),
        program_name: panel_run.program.clone(),
        payloads: Some(payloads)
    }).await?;
    Ok(Arc::new(pro))
}

async fn program(base: PeregrineCoreBase, agent_store: AgentStore, panel: Panel) -> Result<PanelRun,DataMessage> {
    let stick_store = agent_store.stick_store().await;
    let panel_program_store = agent_store.panel_program_store().await;
    match panel.clone().build_panel_run(&stick_store,&panel_program_store).await {
        Ok(r) => Ok(r),
        Err(e) => {
            base.messages.send(e.clone());
            Err(DataMessage::DataMissing(Box::new(e)))
        }
    }
}

#[derive(Clone)]
pub struct PanelRunCache(Agent<PanelRun,Arc<PanelRunOutput>>);

impl PanelRunCache {
    pub fn new(cache_size: usize, base: &PeregrineCoreBase, agent_store: &AgentStore) -> PanelRunCache {
        PanelRunCache(Agent::new(MemoizedType::Cache(cache_size),"panel-run-cache",1,base,agent_store, run))
    }

    pub async fn get(&self, panel_run: &PanelRun) -> Result<Arc<PanelRunOutput>,DataMessage> {
        self.0.get(panel_run).await.as_ref().clone()
    }
}

#[derive(Clone)]
pub struct PanelPrograms(Agent<Panel,PanelRun>);

impl PanelPrograms {
    pub fn new(cache_size: usize, base: &PeregrineCoreBase, agent_store: &AgentStore) -> PanelPrograms {
        PanelPrograms(Agent::new(MemoizedType::Cache(cache_size),"panel-programs",1,base,agent_store, program))
    }

    pub async fn get(&self, panel: &Panel) -> Arc<Result<PanelRun,DataMessage>> {
        self.0.get(panel).await
    }
}

#[derive(Clone)]
pub struct PanelRunStore {
    store: PanelRunCache,
    programs: PanelPrograms
}

impl PanelRunStore {
    pub fn new(cache_size: usize, base: &PeregrineCoreBase, agent_store: &AgentStore) -> PanelRunStore {
        PanelRunStore {
            store: PanelRunCache::new(cache_size,&base,&agent_store),
            programs: PanelPrograms::new(cache_size,&base,&agent_store)
        }
    }
    
    pub async fn run(&self, panel: &Panel) -> Result<Arc<PanelRunOutput>,DataMessage> {
        match self.programs.get(&panel).await.as_ref() {
            Ok(panel_run) => {
                self.store.get(&panel_run).await
            },
            Err(e) => {
                Err(e.clone())
            }
        }
    }
}
