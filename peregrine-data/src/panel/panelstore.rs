use std::sync::{ Arc };
use crate::run::{ PgCommander, PgCommanderTaskSpec, add_task };
use crate::shape::ShapeOutput;
use crate::util::memoized::Memoized;
use super::panel::Panel;
use super::panelrunstore::PanelRunStore;
use crate::util::message::DataMessage;
use crate::api::{ MessageSender, PeregrineCoreBase, AgentStore };

#[derive(Clone)]
pub struct PanelStore {
    store: Memoized<Panel,Result<ShapeOutput,DataMessage>>
}

impl PanelStore {
    pub fn new(cache_size: usize, base: &PeregrineCoreBase, agent_store: &AgentStore) -> PanelStore {
        let agent_store = agent_store.clone();
        let base = base.clone();
        PanelStore {
            store: Memoized::new_cache(cache_size, move |panel: &Panel, result| {
                let agent_store = agent_store.clone();
                let panel = panel.clone();
                add_task(&base.commander,PgCommanderTaskSpec {
                    name: format!("panel {:?}",panel),
                    prio: 1,
                    slot: None,
                    timeout: None,
                    task: Box::pin(async move {
                        let shapes = match agent_store.panel_run_store().await.run(&panel).await {
                            Ok(pro) => {
                                Ok(pro.shapes().filter(panel.min_value() as f64,panel.max_value() as f64))
                            },
                            Err(e) => {
                                Err(DataMessage::DataMissing(Box::new(e.clone())))
                            }
                        };
                        result.resolve(shapes);
                        Ok(())
                    })
                });
            })
        }
    }

    pub async fn run(&self, panel: &Panel) -> Arc<Result<ShapeOutput,DataMessage>> {
        self.store.get(panel).await
    }
}