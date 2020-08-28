use anyhow::bail;
use crate::lock;
use std::collections::HashMap;
use std::sync::{ Arc, Mutex };
use crate::request::Channel;
use crate::run::{ PgDauphin, PgDauphinTaskSpec };
use crate::util::CountingPromise;
use super::panel::PanelSliceRange;
use super::panelprogramstore::PanelProgramStore;
use crate::request::program::ProgramLoader;

pub struct PanelProgramData {
    store: PanelProgramStore
}

impl PanelProgramData {
    fn new() -> PanelProgramData {
        PanelProgramData {
            store: PanelProgramStore::new()
        }
    }

    fn register(&mut self, panel_slice_range: &PanelSliceRange, channel: &Channel, name: &str) {
        self.store.add(panel_slice_range,channel,name);
    }

    fn get_program(&self, panel_slice_range: &PanelSliceRange) -> Option<(Channel,String)> {
        self.store.get(panel_slice_range)
    }
}

#[derive(Clone)]
pub struct PanelProgram {
    dauphin: PgDauphin,
    loader: ProgramLoader,
    booted: CountingPromise,
    data: Arc<Mutex<PanelProgramData>>
}

impl PanelProgram {
    pub fn new(dauphin: &PgDauphin, loader: &ProgramLoader, booted: &CountingPromise) -> PanelProgram {
        PanelProgram {
            dauphin: dauphin.clone(),
            loader: loader.clone(),
            booted: booted.clone(),
            data: Arc::new(Mutex::new(PanelProgramData::new()))
        }
    }

    pub fn register(&self, panel_slice_range: &PanelSliceRange, channel: &Channel, name: &str) {
        lock!(self.data).register(panel_slice_range,channel,name);
        self.loader.load_background(channel,name).unwrap_or(());
    }

    pub async fn run(&self, panel_slice_range: &PanelSliceRange) -> anyhow::Result<()> {
        self.booted.wait().await;
        let program = lock!(self.data).get_program(panel_slice_range);
        if let Some((channel,name)) = program {
            self.dauphin.run_program(&self.loader,PgDauphinTaskSpec {
                prio: 1,
                slot: None,
                timeout: None,
                channel: channel,
                program_name: name,
                payloads: Some(HashMap::new())
            }).await?;
            Ok(())
        } else {
            // XXX should be no-op, I think but this is good during development
            bail!("No panel defined for this range");
        }
    }
}