use std::any::Any;
use std::collections::HashMap;
use crate::lock;
use std::sync::{ Arc };
use crate::index::StickAuthorityStore;
use crate::request::channel::{ Channel, PacketPriority, ChannelIntegration };
use crate::request::manager::{ RequestManager, PayloadReceiver };
use crate::ProgramLoader;
use crate::run::{ PgCommander, PgDauphin, PgCommanderTaskSpec, PgDauphinTaskSpec, add_task };
use crate::shape::ShapeOutput;
use crate::util::memoized::Memoized;
use crate::CountingPromise;
use super::panel::Panel;
use super::programdata::ProgramData;
use crate::index::StickStore;
use super::panelprogramstore::PanelProgramStore;
pub use crate::util::message::DataMessage;
use crate::api::MessageSender;

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

async fn run(booted: CountingPromise, dauphin: PgDauphin, loader: ProgramLoader, panel_run: &PanelRun) -> Result<PanelRunOutput,DataMessage> {
    booted.wait().await;
    let mut payloads = HashMap::new();
    let pro = PanelRunOutput::new();
    payloads.insert("panel".to_string(),Box::new(panel_run.panel.clone()) as Box<dyn Any>);
    payloads.insert("out".to_string(),Box::new(pro.clone()) as Box<dyn Any>);
    payloads.insert("data".to_string(),Box::new(ProgramData::new()) as Box<dyn Any>);
    dauphin.run_program(&loader,PgDauphinTaskSpec {
        prio: 1,
        slot: None,
        timeout: None,
        channel: panel_run.channel.clone(),
        program_name: panel_run.program.clone(),
        payloads: Some(payloads)
    }).await.map_err(|e| DataMessage::XXXTmp(e.to_string()))?;
    Ok(pro)
}

#[derive(Clone)]
pub struct PanelRunStore {
    stick_store: StickStore,
    panel_program_store: PanelProgramStore,
    store: Memoized<PanelRun,PanelRunOutput>,
    programs: Memoized<Panel,Result<PanelRun,DataMessage>>
}

impl PanelRunStore {
    pub fn new(cache_size: usize, commander: &PgCommander, dauphin: &PgDauphin, loader: &ProgramLoader, 
                stick_store: &StickStore, panel_program_store: &PanelProgramStore, messages: &MessageSender, booted: &CountingPromise) -> PanelRunStore {
        let commander = commander.clone();
        let commander2 = commander.clone();
        let booted = booted.clone();
        let loader = loader.clone();
        let dauphin = dauphin.clone();
        let stick_store = stick_store.clone();
        let panel_program_store = panel_program_store.clone();
        let stick_store2 = stick_store.clone();
        let panel_program_store2 = panel_program_store.clone();
        let messages = messages.clone();
        PanelRunStore {
            stick_store: stick_store,
            panel_program_store: panel_program_store,
            store: Memoized::new_cache(cache_size, move |panel_run: &PanelRun, result| {
                let booted = booted.clone();
                let dauphin = dauphin.clone();
                let loader = loader.clone();
                let panel_run = panel_run.clone();
                add_task(&commander,PgCommanderTaskSpec {
                    name: format!("panel run for: {:?}",panel_run),
                    prio: 1,
                    slot: None,
                    timeout: None,
                    task: Box::pin(async move {
                        result.resolve(run(booted.clone(),dauphin.clone(),loader.clone(),&panel_run).await?);
                        Ok(())
                    })
                });
            }),
            programs: Memoized::new_cache(cache_size, move |panel: &Panel,result| {
                let panel = panel.clone();
                let stick_store = stick_store2.clone();
                let panel_program_store = panel_program_store2.clone();
                let messages = messages.clone();
                add_task(&commander2,PgCommanderTaskSpec {
                    name: format!("panel build run for: {:?}",panel),
                    prio: 1,
                    slot: None,
                    timeout: None,
                    task: Box::pin(async move {
                        let r = match panel.clone().build_panel_run(&stick_store,&panel_program_store).await {
                            Ok(r) => Ok(r),
                            Err(e) => {
                                messages.send(e.clone());
                                Err(DataMessage::DataMissing(Box::new(e)))
                            }
                        };
                        result.resolve(r);
                        Ok(())
                    })
                });
            })
        }
    }

    pub async fn run(&self, panel: &Panel) -> Result<Arc<PanelRunOutput>,DataMessage> {
        match self.programs.get(&panel).await.as_ref() {
            Ok(panel_run) => {
                Ok(self.store.get(&panel_run).await)
            },
            Err(e) => {
                Err(e.clone())
            }
        }
    }
}
