use std::sync::{ Arc };
use crate::run::{ PgCommander, PgCommanderTaskSpec, add_task };
use crate::shape::ShapeOutput;
use crate::util::memoized::Memoized;
use super::panel::Panel;
use super::panelrunstore::PanelRunStore;
use crate::util::message::DataMessage;
use crate::api::MessageSender;

#[derive(Clone)]
pub struct PanelStore {
    store: Memoized<Panel,Result<ShapeOutput,DataMessage>>
}

impl PanelStore {
    pub fn new(cache_size: usize, commander: &PgCommander, panel_run_store: &PanelRunStore, messages: &MessageSender) -> PanelStore {
        let panel_run_store = panel_run_store.clone();
        let commander = commander.clone();
        let messages = messages.clone();
        PanelStore {
            store: Memoized::new_cache(cache_size, move |panel: &Panel, result| {
                let panel_run_store = panel_run_store.clone();
                let commander = commander.clone();
                let panel = panel.clone();
                let messages = messages.clone();
                add_task(&commander,PgCommanderTaskSpec {
                    name: format!("panel {:?}",panel),
                    prio: 1,
                    slot: None,
                    timeout: None,
                    task: Box::pin(async move {
                        let shapes = match panel_run_store.run(&panel).await {
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