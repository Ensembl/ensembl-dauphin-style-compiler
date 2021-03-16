use std::any::Any;
use std::collections::HashMap;
use std::sync::{ Arc };
use crate::request::channel::{ Channel, PacketPriority, ChannelIntegration };
use crate::request::manager::{ RequestManager, PayloadReceiver };
use crate::ProgramLoader;
use crate::run::{ PgCommander, PgDauphin, PgCommanderTaskSpec, PgDauphinTaskSpec, add_task, async_complete_task };
use crate::shape::ShapeOutput;
use crate::util::memoized::Memoized;
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

async fn run(base: PeregrineCoreBase, agent_store: AgentStore, panel_run: &PanelRun) -> Result<PanelRunOutput,DataMessage> {
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
    Ok(pro)
}

#[derive(Clone)]
pub struct PanelRunStore {
    store: Memoized<PanelRun,Result<Arc<PanelRunOutput>,DataMessage>>,
    programs: Memoized<Panel,Result<PanelRun,DataMessage>>
}

impl PanelRunStore {
    pub fn new(cache_size: usize, base: &PeregrineCoreBase, agent_store: &AgentStore) -> PanelRunStore {
        let base = base.clone();
        let base2 = base.clone();
        let agent_store = agent_store.clone();
        let agent_store2 = agent_store.clone();
        PanelRunStore {
            store: Memoized::new_cache(cache_size, move |panel_run: &PanelRun, result| {
                let base = base2.clone();
                let agent_store = agent_store.clone();
                let panel_run = panel_run.clone();
                let handle = add_task(&base2.commander,PgCommanderTaskSpec {
                    name: format!("panel run for: {:?}",panel_run),
                    prio: 1,
                    slot: None,
                    timeout: None,
                    task: Box::pin(async move {
                        result.resolve(run(base,agent_store,&panel_run).await.map(|x| Arc::new(x)));
                        Ok(())
                    })
                });
                async_complete_task(&base2.commander,&base2.messages,handle,|e| (e,false));
            }),
            programs: Memoized::new_cache(cache_size, move |panel: &Panel,result| {
                let panel = panel.clone();
                let agent_store = agent_store2.clone();
                let base2 = base.clone();
                let handle = add_task(&base.commander,PgCommanderTaskSpec {
                    name: format!("panel build run for: {:?}",panel),
                    prio: 1,
                    slot: None,
                    timeout: None,
                    task: Box::pin(async move {
                        let stick_store = agent_store.stick_store().await;
                        let panel_program_store = agent_store.panel_program_store().await;
                        let r = match panel.clone().build_panel_run(&stick_store,&panel_program_store).await {
                            Ok(r) => Ok(r),
                            Err(e) => {
                                base2.messages.send(e.clone());
                                Err(DataMessage::DataMissing(Box::new(e)))
                            }
                        };
                        result.resolve(r);
                        Ok(())
                    })
                });
                async_complete_task(&base.commander, &base.messages,handle,|e| (e,false));
            })
        }
    }

    pub async fn run(&self, panel: &Panel) -> Result<Arc<PanelRunOutput>,DataMessage> {
        match self.programs.get(&panel).await.as_ref() {
            Ok(panel_run) => {
                self.store.get(&panel_run).await.as_ref().clone()
            },
            Err(e) => {
                Err(e.clone())
            }
        }
    }
}
