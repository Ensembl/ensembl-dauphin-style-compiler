use std::any::Any;
use std::collections::HashMap;
use crate::lock;
use std::sync::{ Arc, Mutex };
use crate::index::StickAuthorityStore;
use crate::request::channel::{ Channel, PacketPriority, ChannelIntegration };
use crate::request::manager::{ RequestManager, PayloadReceiver };
use crate::ProgramLoader;
use crate::run::{ PgCommander, PgDauphin, PgCommanderTaskSpec, PgDauphinTaskSpec };
use crate::shape::ShapeZoo;
use crate::util::memoized::Memoized;
use crate::CountingPromise;
use super::panel::Panel;
use crate::index::StickStore;
use super::panelprogramstore::PanelProgramStore;

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
    zoo: ShapeZoo
}

impl PanelRunOutput {
    fn new() -> PanelRunOutput {
        PanelRunOutput {
            zoo: ShapeZoo::new()
        }
    }

    pub fn zoo(&self) -> &ShapeZoo { &self.zoo }
}

async fn run(booted: CountingPromise, dauphin: PgDauphin, loader: ProgramLoader, panel_run: &PanelRun) -> anyhow::Result<PanelRunOutput> {
    booted.wait().await;
    let mut payloads = HashMap::new();
    let pro = PanelRunOutput::new();
    payloads.insert("out".to_string(),Box::new(pro.clone()) as Box<dyn Any>);
    dauphin.run_program(&loader,PgDauphinTaskSpec {
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
    stick_store: StickStore,
    panel_program_store: PanelProgramStore,
    store: Memoized<PanelRun,PanelRunOutput>
}

impl PanelRunStore {
    pub fn new(cache_size: usize, commander: &PgCommander, dauphin: &PgDauphin, loader: &ProgramLoader, 
                stick_store: &StickStore, panel_program_store: &PanelProgramStore, booted: &CountingPromise) -> PanelRunStore {
        let commander = commander.clone();
        let booted = booted.clone();
        let loader = loader.clone();
        let dauphin = dauphin.clone();
        let stick_store = stick_store.clone();
        let panel_program_store = panel_program_store.clone();
        PanelRunStore {
            stick_store: stick_store,
            panel_program_store: panel_program_store,
            store: Memoized::new_cache(cache_size, move |panel_run: &PanelRun, result| {
                let booted = booted.clone();
                let dauphin = dauphin.clone();
                let loader = loader.clone();
                let panel_run = panel_run.clone();
                commander.add_task(PgCommanderTaskSpec {
                    name: format!("panel run for: {:?}",panel_run),
                    prio: 1,
                    slot: None,
                    timeout: None,
                    task: Box::pin(async move {
                        result.resolve(run(booted.clone(),dauphin.clone(),loader.clone(),&panel_run).await?);
                        Ok(())
                    })
                });
            })
        }
    }

    pub async fn run(&self, panel: &Panel) -> anyhow::Result<Arc<PanelRunOutput>> {
        let panel_run = panel.build_panel_run(&self.stick_store,&self.panel_program_store).await?;
        let output = self.store.get(&panel_run).await?;
        if panel.scale() != panel_run.panel.scale() {
            Ok(Arc::new(PanelRunOutput {
                zoo: output.zoo.filter(panel.min_value() as f64,panel.max_value() as f64)
            }))
        } else {
            Ok(output)
        }
    }
}