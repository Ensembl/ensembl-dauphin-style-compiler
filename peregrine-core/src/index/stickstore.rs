
use crate::lock;
use std::sync::{ Arc, Mutex };
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use crate::core::stick::{ Stick, StickId, StickTopology };
use anyhow::bail;
use crate::util::singlefile::SingleFile;
use crate::request::stick::get_stick;
use crate::request::channel::{ Channel, PacketPriority, ChannelIntegration };
use crate::request::manager::{ RequestManager, PayloadReceiver };
use crate::request::packet::ResponsePacket;
use crate::run::{ PgCommander, PgDauphin };
use crate::run::pgcommander::PgCommanderTaskSpec;

#[derive(Clone)]
pub struct StickStore {
    single_file: SingleFile<StickId,()>,
    store: Arc<Mutex<HashMap<StickId,Option<Stick>>>>
}

impl StickStore {
    pub fn new(commander: &PgCommander, manager: &RequestManager, channel: &Channel) -> anyhow::Result<StickStore> {
        let store = Arc::new(Mutex::new(HashMap::new()));
        let channel = channel.clone();
        let manager2 = manager.clone();
        let store2 = store.clone();
        Ok(StickStore {
            single_file: SingleFile::new(commander,move |stick_id : &StickId| {
                let manager = manager2.clone();
                let stick_id = stick_id.clone();
                let channel = channel.clone();
                let store = store2.clone();
                PgCommanderTaskSpec {
                    name: format!("stick-loader-{}-{}",channel,stick_id.get_id()),
                    prio: 3,
                    timeout: None,
                    slot: None,
                    task: Box::pin(async move {
                        let stick = get_stick(manager,channel.clone(),stick_id.clone()).await.ok();
                        lock!(store).insert(stick_id.clone(),stick);
                        Ok(())
                    })
                }
            }),
            store
        })
    }

    pub(crate) fn add(&self, id: &StickId, stick: Option<&Stick>) {
        lock!(self.store).insert(id.clone(),stick.cloned());
    }

    pub async fn lookup(&self, id: &StickId) -> anyhow::Result<Option<Stick>> {
        let stick = lock!(self.store).get(id).cloned();
        if let Some(stick) = stick {
            Ok(stick.clone())
        } else {
            self.single_file.request(id.clone()).await?;
            if let Some(stick) = lock!(self.store).get(id) {
                Ok(stick.clone())
            } else {
                bail!(format!("unexpected failure retrieving stick {}",id.get_id()));
            }
        }
    }
}
