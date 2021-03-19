use std::sync::{ Arc };
use crate::{ agent::agent::Agent};
use crate::shape::ShapeOutput;
use crate::util::memoized::{ MemoizedType };
use super::panel::Panel;
use crate::util::message::DataMessage;
use crate::api::{ PeregrineCoreBase, AgentStore };

async fn run(_base: PeregrineCoreBase,agent_store: AgentStore, panel: Panel) -> Result<ShapeOutput,DataMessage> {
    match agent_store.panel_run_store().await.run(&panel).await {
        Ok(pro) => {
            Ok(pro.shapes().filter(panel.min_value() as f64,panel.max_value() as f64))
        },
        Err(e) => {
            Err(DataMessage::DataMissing(Box::new(e.clone())))
        }
    }
}

#[derive(Clone)]
pub struct PanelStore(Agent<Panel,ShapeOutput>);

impl PanelStore {
    pub fn new(cache_size: usize, base: &PeregrineCoreBase, agent_store: &AgentStore) -> PanelStore {
        PanelStore(Agent::new(MemoizedType::Cache(cache_size),"panel",1,base,agent_store, run))
    }

    pub async fn run(&self, panel: &Panel) -> Arc<Result<ShapeOutput,DataMessage>> {
        self.0.get(panel).await
    }
}
