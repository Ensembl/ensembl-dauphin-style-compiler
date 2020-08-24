
use crate::lock;
use std::sync::{ Arc, Mutex };
use std::collections::HashMap;
use crate::core::stick::{ Stick, StickId, StickTopology };
use anyhow::bail;
use crate::util::singlefile::SingleFile;
use crate::request::stick::get_stick;
use crate::request::channel::{ Channel, PacketPriority, ChannelIntegration };
use crate::request::manager::{ RequestManager, PayloadReceiver };
use crate::request::packet::ResponsePacket;
use crate::run::{ PgCommander, PgDauphin };
use crate::run::pgcommander::PgCommanderTaskSpec;

struct StickLoaderData {
    single_file: SingleFile<StickId,()>
}

#[derive(Clone)]
pub struct StickLoader(Arc<Mutex<StickLoaderData>>);

impl StickLoader {
    pub fn new(commander: &PgCommander, manager: &RequestManager, channel: &Channel) -> anyhow::Result<StickLoader> {
        let manager2 = manager.clone();
        let channel = channel.clone();
        let out = StickLoader(Arc::new(Mutex::new(StickLoaderData {
            single_file: SingleFile::new(commander,move |stick_id : &StickId| {
                let manager = manager2.clone();
                PgCommanderTaskSpec {
                    name: format!("stick-loader-{}-{}",channel,stick_id.get_id()),
                    prio: 3,
                    timeout: None,
                    slot: None,
                    task: Box::pin(get_stick(manager,channel.clone(),stick_id.clone()))
                }
            })
        })));
        Ok(out)
    }

    pub async fn load(&self, stick_id: &StickId) -> anyhow::Result<()> {
        lock!(self.0).single_file.request(stick_id.clone()).await
    }
}

#[derive(Clone)]
pub struct StickStore {
    loader: StickLoader,
    store: Arc<Mutex<HashMap<StickId,Stick>>>
}

impl StickStore {
    pub fn new(commander: &PgCommander, manager: &RequestManager, channel: &Channel) -> anyhow::Result<StickStore> {
        Ok(StickStore {
            loader: StickLoader::new(commander,manager,channel)?,
            store: Arc::new(Mutex::new(HashMap::new()))
        })
    }

    pub(crate) fn add(&self, stick: &Stick) {
        lock!(self.store).insert(stick.get_id().clone(),stick.clone());
    }

    pub async fn lookup(&self, id: &StickId) -> anyhow::Result<Stick> {
        let stick = lock!(self.store).get(id).cloned();
        if let Some(stick) = stick {
            Ok(stick.clone())
        } else {
            self.loader.load(id).await?;
            if let Some(stick) = lock!(self.store).get(id) {
                Ok(stick.clone())
            } else {
                bail!(format!("unexpected failure retrieving stick {}",id.get_id()));
            }
        }
    }
}

impl PayloadReceiver for StickStore {
    fn receive(&self, channel: &Channel, response: &ResponsePacket, channel_itn: &ChannelIntegration) {
        for stick in response.sticks().iter() {
            self.add(stick);
        }
    }
}
