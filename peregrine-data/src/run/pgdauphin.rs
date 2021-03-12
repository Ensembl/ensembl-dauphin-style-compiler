use anyhow::{ self, anyhow as err };
use blackbox::blackbox_log;
use crate::lock;
use commander::{ RunSlot };
use serde_cbor::Value as CborValue;
use std::any::Any;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::{ Arc, Mutex };
use crate::api::MessageSender;
use crate::request::channel::{ Channel, ChannelIntegration };
use crate::request::manager::{ PayloadReceiver };
use crate::request::packet::ResponsePacket;
use crate::request::program::ProgramLoader;
use peregrine_dauphin_queue::{ PgDauphinQueue, PgDauphinLoadTaskSpec, PgDauphinRunTaskSpec };

pub struct PgDauphinTaskSpec {
    pub prio: i8, 
    pub slot: Option<RunSlot>, 
    pub timeout: Option<f64>,
    pub channel: Channel,
    pub program_name: String,
    pub payloads: Option<HashMap<String,Box<dyn Any>>>
}

struct PgDauphinData {
    pdq: PgDauphinQueue,
    names: HashMap<(String,String),Option<(String,String)>>,
}

#[derive(Clone)]
pub struct PgDauphin(Arc<Mutex<PgDauphinData>>);


impl PgDauphinData {
    
}

impl PgDauphin {
    pub fn new(pdq: &PgDauphinQueue) -> anyhow::Result<PgDauphin> {
        Ok(PgDauphin(Arc::new(Mutex::new(PgDauphinData {
            pdq: pdq.clone(),
            names: HashMap::new(),
        }))))
    }

    pub async fn add_binary_direct(&self, binary_name: &str, cbor: &CborValue) -> anyhow::Result<()> {
        lock!(self.0).pdq.load(PgDauphinLoadTaskSpec {
            bundle_name: binary_name.to_string(),
            data: cbor.clone()
        }).await
    }

    fn binary_name(&self, channel: &Channel, name_of_bundle: &str) -> String {
        let channel_name = channel.to_string();
        format!("{}-{}-{}",channel_name.len(),channel_name,name_of_bundle)
    }

    pub async fn add_binary(&self, channel: &Channel, name_of_bundle: &str, cbor: &CborValue) -> anyhow::Result<()> {
        self.add_binary_direct(&self.binary_name(channel,name_of_bundle),cbor).await
    }

    pub async fn add_programs(&self, channel: &Channel, response: &ResponsePacket) -> anyhow::Result<()> {
        for bundle in response.programs().iter() {
            blackbox_log!(&format!("channel-{}",channel.to_string()),"registered bundle {}",bundle.bundle_name());
            self.add_binary(channel,bundle.bundle_name(),bundle.program()).await?;
            for (in_channel_name,in_bundle_name) in bundle.name_map() {
                blackbox_log!(&format!("channel-{}",channel.to_string()),"registered program {}",in_channel_name);
                self.register(channel,in_channel_name,&self.binary_name(channel,bundle.bundle_name()),in_bundle_name);
            }
        }
        Ok(())
    }

    pub fn register(&self, channel: &Channel, name_in_channel: &str, name_of_bundle: &str, name_in_bundle: &str) {
        let binary_name = self.binary_name(channel,name_of_bundle);
        lock!(self.0).names.insert((channel.to_string(),name_in_channel.to_string()),Some((binary_name,name_in_bundle.to_string())));
    }

    pub fn is_present(&self, channel: &Channel, name_in_channel: &str) -> bool {
        lock!(self.0).names.get(&(channel.to_string(),name_in_channel.to_string())).and_then(|x| x.as_ref()).is_some()
    }

    pub fn mark_missing(&self, channel: &Channel, name_in_channel: &str) {
        let mut data = lock!(self.0);
        data.names.insert((channel.channel_name(),name_in_channel.to_string()),None);
    }

    pub async fn run_program(&self, loader: &ProgramLoader, spec: PgDauphinTaskSpec) -> anyhow::Result<()> {
        if !self.is_present(&spec.channel,&spec.program_name) {
            loader.load(&spec.channel,&spec.program_name).await?;
        }
        let data = lock!(self.0);
        let (bundle_name,in_bundle_name) = data.names.get(&(spec.channel.to_string(),spec.program_name.to_string())).as_ref().unwrap().as_ref()
            .ok_or(err!("Failed channel/program = {}/{}",spec.channel.to_string(),spec.program_name))?.to_owned();
        let mut payloads = spec.payloads.unwrap_or_else(|| HashMap::new());
        payloads.insert("channel".to_string(),Box::new(spec.channel.clone()));
        let pdq = data.pdq.clone();
        drop(data);
        pdq.run(PgDauphinRunTaskSpec {
            prio: spec.prio,
            slot: spec.slot,
            timeout: spec.timeout,
            bundle_name, in_bundle_name,
            payloads
        }).await
    }
}

impl PayloadReceiver for PgDauphin {
    fn receive(&self, channel: &Channel, response: ResponsePacket, channel_itn: &Rc<Box<dyn ChannelIntegration>>, messages: &MessageSender) -> Pin<Box<dyn Future<Output=ResponsePacket>>> {
        let pgd = self.clone();
        let channel = channel.clone();
        let channel_itn = channel_itn.clone();
        let messages = messages.clone();
        Box::pin(async move {
            for bundle in response.programs().clone().iter() {
                match pgd.add_binary(&channel,bundle.bundle_name(),bundle.program()).await {
                    Ok(_) => {
                        for (in_channel_name,in_bundle_name) in bundle.name_map() {
                            pgd.register(&channel,in_channel_name,bundle.bundle_name(),in_bundle_name);
                        }
                    },
                    Err(e) => {
                        messages.send(&format!("error: {:?}",e));
                        for (in_channel_name,_) in bundle.name_map() {
                            pgd.mark_missing(&channel,in_channel_name);
                        }
                    }
                }
            }
            response
        })
    }
}
