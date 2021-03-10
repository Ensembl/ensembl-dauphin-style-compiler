use std::sync::{ Arc };
use crate::run::{ PgCommander, PgCommanderTaskSpec };
use crate::util::memoized::Memoized;
use super::panel::Panel;
use super::panelrunstore::PanelRunStore;
use crate::{ Channel, RequestManager };
use crate::request::data::{ DataCommandRequest, DataResponse };

#[derive(Clone)]
pub struct DataStore {
    store: Memoized<(Panel,Channel,String),Box<DataResponse>>
}

impl DataStore {
    pub fn new(cache_size: usize, commander: &PgCommander, manager: &RequestManager) -> DataStore {
        let manager = manager.clone();
        let commander = commander.clone();
        DataStore {
            store: Memoized::new_cache(cache_size, move |data: &(Panel,Channel,String), result| {
                let (panel,channel,name) = data;
                let manager = manager.clone();
                let commander = commander.clone();
                let panel = panel.clone();
                let channel = channel.clone();
                let name = name.clone();
                commander.add_task(PgCommanderTaskSpec {
                    name: format!("data for panel {:?}",panel),
                    prio: 1,
                    slot: None,
                    timeout: None,
                    task: Box::pin(async move {
                        let data_command_request = DataCommandRequest::new(&channel,&name,&panel);
                        let out = data_command_request.execute(manager).await?;
                        result.resolve(out);
                        Ok(())
                    })
                });
            })
        }
    }

    pub async fn get(&self, panel: &Panel, channel: &Channel, name: &str) -> anyhow::Result<Arc<Box<DataResponse>>> {
        self.store.get(&(panel.clone(),channel.clone(),name.to_string())).await
    }
}