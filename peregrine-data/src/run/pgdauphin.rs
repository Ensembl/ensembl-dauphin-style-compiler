use anyhow::{ self, anyhow as err };
use peregrine_toolkit::lock;
use commander::{ RunSlot };
use std::any::Any;
use std::collections::HashMap;
use std::sync::{ Arc, Mutex };
use crate::core::channel::channelregistry::ChannelRegistry;
use crate::{MaxiResponse, BackendNamespace, AccessorResolver};
use crate::api::MessageSender;
use crate::core::programbundle::SuppliedBundle;
use crate::shapeload::programloader::ProgramLoader;
use crate::util::message::DataMessage;
use peregrine_dauphin_queue::{ PgDauphinQueue, PgDauphinLoadTaskSpec, PgDauphinRunTaskSpec };
use crate::shapeload::programname::ProgramName;

pub struct PgDauphinTaskSpec {
    pub prio: u8, 
    pub slot: Option<RunSlot>, 
    pub timeout: Option<f64>,
    pub program_name: ProgramName,
    pub payloads: Option<HashMap<String,Box<dyn Any>>>
}

struct PgDauphinData {
    pdq: PgDauphinQueue,
    names: HashMap<ProgramName,Option<(String,String)>>,
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

    async fn add_binary_direct(&self, binary_name: &str, data: &[u8]) -> anyhow::Result<()> {
        lock!(self.0).pdq.load(PgDauphinLoadTaskSpec {
            bundle_name: binary_name.to_string(),
            data: data.to_vec()
        }).await
    }

    fn binary_name(&self, channel: &BackendNamespace, name_of_bundle: &str) -> String {
        let channel_name = channel.to_string();
        format!("{}-{}-{}",channel_name.len(),channel_name,name_of_bundle)
    }

    async fn add_binary(&self, channel: &BackendNamespace, name_of_bundle: &str, cbor: &[u8]) -> anyhow::Result<()> {
        self.add_binary_direct(&self.binary_name(channel,name_of_bundle),cbor).await
    }

    pub async fn add_programs(&self, channel: &BackendNamespace, response: &MaxiResponse) -> anyhow::Result<()> {
        for bundle in response.programs().iter() {
            self.add_binary(channel,bundle.bundle_name(),bundle.program()).await?;
            for (in_channel_name,in_bundle_name) in bundle.name_map() {
                self.register(&ProgramName(channel.clone(),in_channel_name.to_string()),&self.binary_name(channel,bundle.bundle_name()),in_bundle_name);
            }
        }
        Ok(())
    }

    pub fn register(&self, program_name: &ProgramName, name_of_bundle: &str, name_in_bundle: &str) {
        let binary_name = self.binary_name(&program_name.0,name_of_bundle);
        lock!(self.0).names.insert(program_name.clone(),Some((binary_name,name_in_bundle.to_string())));
    }

    pub fn is_present(&self, program_name: &ProgramName) -> bool {
        lock!(self.0).names.get(program_name).and_then(|x| x.as_ref()).is_some()
    }

    pub fn mark_missing(&self, program_name: &ProgramName) {
        let mut data = lock!(self.0);
        data.names.insert(program_name.clone(),None);
    }

    pub async fn run_program(&self, loader: &ProgramLoader, registry: &ChannelRegistry, spec: PgDauphinTaskSpec) -> Result<(),DataMessage> {
        let program_name = spec.program_name.clone();
        if !self.is_present(&program_name) {
            loader.load(&program_name).await.map_err(|e| DataMessage::XXXTransitional(e))?;
        }
        let data = lock!(self.0);
        let (bundle_name,in_bundle_name) = data.names.get(&program_name).as_ref().unwrap().as_ref()
            .ok_or(err!("Failed channel/program = {}",spec.program_name.to_string())).map_err(|e| DataMessage::DauphinProgramMissing(e.to_string()))?.to_owned();
        let mut payloads = spec.payloads.unwrap_or_else(|| HashMap::new());
        payloads.insert("channel".to_string(),Box::new(spec.program_name.0.clone()));
        payloads.insert("channel-resolver".to_string(),Box::new(AccessorResolver::new(registry,&spec.program_name.0)));
        let pdq = data.pdq.clone();
        drop(data);
        pdq.run(PgDauphinRunTaskSpec {
            prio: spec.prio,
            slot: spec.slot,
            timeout: spec.timeout,
            bundle_name, in_bundle_name,
            payloads
        }).await.map_err(|e| DataMessage::DauphinRunError(program_name.clone(),e.to_string()))
    }
}

async fn add_bundle(pgd: &PgDauphin, channel: &BackendNamespace, bundle: &SuppliedBundle, messages: &MessageSender) {
    match pgd.add_binary(&channel,bundle.bundle_name(),bundle.program()).await {
        Ok(_) => {
            for (in_channel_name,in_bundle_name) in bundle.name_map() {
                pgd.register(&ProgramName(channel.clone(),in_channel_name.to_string()),bundle.bundle_name(),in_bundle_name);
            }
        },
        Err(e) => {
            messages.send(DataMessage::BadDauphinProgram(format!("{:#}",e)));
            for (in_channel_name,_) in bundle.name_map() {
                pgd.mark_missing(&ProgramName(channel.clone(),in_channel_name.to_string()));
            }
        }
    }
}

pub(crate) async fn add_programs_from_response(pgd: &PgDauphin, channel: &BackendNamespace, response: &MaxiResponse, messages: &MessageSender) {
    for bundle in response.programs().clone().iter() {
        add_bundle(&pgd,&channel, bundle, &messages).await;
    }
}
