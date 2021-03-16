use std::sync::{ Arc };
use crate::api::{ MessageSender, PeregrineCoreBase };
use crate::run::{ PgCommander, PgCommanderTaskSpec, add_task, async_complete_task };
use crate::util::memoized::Memoized;
use crate::util::message::{ DataMessage };
use super::panel::Panel;
use crate::{ Channel, RequestManager };
use crate::request::data::{ DataCommandRequest, DataResponse };

// TODO Memoized errors with retry semantics

#[derive(Clone)]
pub struct DataStore {
    store: Memoized<(Panel,Channel,String),Result<Arc<Box<DataResponse>>,DataMessage>>
}

impl DataStore {
    pub fn new(cache_size: usize, base: &PeregrineCoreBase) -> DataStore {
        let base = base.clone();
        DataStore {
            store: Memoized::new_cache(cache_size, move |data: &(Panel,Channel,String), result| {
                let (panel,channel,name) = data;
                let base2 = base.clone();
                let panel = panel.clone();
                let channel = channel.clone();
                let name = name.clone();
                let handle = add_task(&base.commander,PgCommanderTaskSpec {
                    name: format!("data for panel {:?}",panel),
                    prio: 1,
                    slot: None,
                    timeout: None,
                    task: Box::pin(async move {
                        let data_command_request = DataCommandRequest::new(&channel,&name,&panel);
                        let r = data_command_request.execute(base2.manager).await.map(|x| Arc::new(x));
                        result.resolve(r);
                        Ok(())
                    })
                });
                async_complete_task(&base.commander,&base.messages,handle,|e| (e,false));
            })
        }
    }

    pub async fn get(&self, panel: &Panel, channel: &Channel, name: &str) -> Result<Arc<Box<DataResponse>>,DataMessage> {
        self.store.get(&(panel.clone(),channel.clone(),name.to_string())).await.as_ref().clone()
    }
}