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
use super::panelrunstore::PanelRunStore;
use web_sys::console;

#[derive(Clone)]
pub struct PanelStore {
    store: Memoized<Panel,ShapeZoo>
}

impl PanelStore {
    pub fn new(cache_size: usize, commander: &PgCommander, panel_run_store: &PanelRunStore) -> PanelStore {
        let panel_run_store = panel_run_store.clone();
        let commander = commander.clone();
        PanelStore {
            store: Memoized::new_cache(cache_size, move |panel: &Panel, result| {
                let panel_run_store = panel_run_store.clone();
                let commander = commander.clone();
                let panel = panel.clone();
                commander.add_task(PgCommanderTaskSpec {
                    name: format!("panel {:?}",panel),
                    prio: 1,
                    slot: None,
                    timeout: None,
                    task: Box::pin(async move {
                        let pro = panel_run_store.run(&panel).await?;
                        let zoo = pro.zoo().filter(panel.min_value() as f64,panel.max_value() as f64);
                        result.resolve(zoo);
                        Ok(())
                    })
                });
            })
        }
    }

    pub async fn run(&self, panel: &Panel) -> anyhow::Result<Arc<ShapeZoo>> {
        self.store.get(panel).await
    }
}