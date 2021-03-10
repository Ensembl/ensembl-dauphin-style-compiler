use std::sync::{ Arc };
use crate::run::{ PgCommander, PgCommanderTaskSpec };
use crate::shape::ShapeOutput;
use crate::util::memoized::Memoized;
use super::panel::Panel;
use super::panelrunstore::PanelRunStore;

#[derive(Clone)]
pub struct PanelStore {
    store: Memoized<Panel,ShapeOutput>
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
                        let shapes = pro.shapes().filter(panel.min_value() as f64,panel.max_value() as f64);
                        result.resolve(shapes);
                        Ok(())
                    })
                });
            })
        }
    }

    pub async fn run(&self, panel: &Panel) -> anyhow::Result<Arc<ShapeOutput>> {
        self.store.get(panel).await
    }
}