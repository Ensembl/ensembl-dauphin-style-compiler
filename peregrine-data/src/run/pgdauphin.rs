use anyhow::{ self };
use peregrine_toolkit::error::Error;
use peregrine_toolkit::lock;
use std::any::Any;
use std::collections::HashMap;
use std::sync::{ Arc, Mutex };
use crate::core::channel::channelregistry::ChannelRegistry;
use crate::core::program::programspec::ProgramModel;
use crate::{MaxiResponse, BackendNamespace, AccessorResolver};
use crate::api::MessageSender;
use crate::core::program::programbundle::SuppliedBundle;
use crate::shapeload::programloader::ProgramLoader;
use peregrine_dauphin_queue::{ PgDauphinQueue, PgDauphinLoadTaskSpec, PgDauphinRunTaskSpec };
use crate::shapeload::programname::{ProgramName};

pub struct PgDauphinTaskSpec {
    pub prio: u8, 
    pub program_name: ProgramName,
    pub payloads: Option<HashMap<String,Box<dyn Any>>>
}

#[derive(Clone)]
struct InternalName {
    backend_namespace: BackendNamespace,
    program: ProgramModel,
    bundle_name: String,
    in_bundle_name: String
}

impl InternalName {
    fn new(backend_namespace: &BackendNamespace, program: &ProgramModel, bundle_name: &str, in_bundle_name: &str) -> InternalName {
        InternalName {
            backend_namespace: backend_namespace.clone(),
            bundle_name: bundle_name.to_string(),
            program: program.clone(),
            in_bundle_name: in_bundle_name.to_string()
        }
    }
}

struct PgDauphinData {
    pdq: PgDauphinQueue,
    names: HashMap<ProgramName,Option<InternalName>>
}

#[derive(Clone)]
pub struct PgDauphin(Arc<Mutex<PgDauphinData>>);

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

    fn register(&self, backend_namespace: &BackendNamespace, program: &ProgramModel, name_of_bundle: &str) {
        let binary_name = self.binary_name(backend_namespace,name_of_bundle);
        let internal_name = InternalName::new(backend_namespace,program,&binary_name,program.in_bundle_name());
        lock!(self.0).names.insert(program.name().clone(),Some(internal_name));
    }

    pub fn is_present(&self, program_name: &ProgramName) -> bool {
        lock!(self.0).names.get(program_name).and_then(|x| x.as_ref()).is_some()
    }

    pub fn mark_missing(&self, program_name: &ProgramName) {
        let mut data = lock!(self.0);
        data.names.insert(program_name.clone(),None);
    }

    pub async fn run_program(&self, loader: &ProgramLoader, registry: &ChannelRegistry, spec: PgDauphinTaskSpec) -> Result<(),Error> {
        let program_name = spec.program_name.clone();
        if !self.is_present(&program_name) {
            loader.load(&program_name).await?;
        }
        let data = lock!(self.0);
        let internal_name = data.names.get(&program_name).as_ref().unwrap().as_ref()
            .ok_or(Error::operr(&format!("failed channel/program {:?}",program_name)))?.clone();
        let mut payloads = spec.payloads.unwrap_or_else(|| HashMap::new());
        payloads.insert("channel-resolver".to_string(),Box::new(AccessorResolver::new(registry,&internal_name.backend_namespace)));
        let pdq = data.pdq.clone();
        drop(data);
        pdq.run(PgDauphinRunTaskSpec {
            prio: spec.prio,
            slot: None,
            timeout: None,
            bundle_name: internal_name.bundle_name.to_string(), 
            in_bundle_name: internal_name.in_bundle_name.to_string(),
            payloads
        }).await.map_err(|e| Error::operr(&format!("Cannot run {:?}: {}",program_name,e)))
    }
}

async fn add_bundle(pgd: &PgDauphin, channel: &BackendNamespace, bundle: &SuppliedBundle, messages: &MessageSender) -> Result<(),Error> {
    let specs = bundle.specs().to_program_models()?;
    match pgd.add_binary(&channel,bundle.bundle_name(),bundle.program()).await {
        Ok(_) => {
            for spec in specs {
                pgd.register(channel,&spec,bundle.bundle_name());
            }
        },
        Err(e) => {
            messages.send(Error::operr(&format!("{:#}",e)));
            for spec in specs {
                pgd.mark_missing(spec.name());
            }
        }
    }
    Ok(())
}

pub(crate) async fn add_programs_from_response(pgd: &PgDauphin, channel: &BackendNamespace, response: &MaxiResponse, messages: &MessageSender) -> Result<(),Error> {
    for bundle in response.programs().clone().iter() {
        add_bundle(&pgd,&channel, bundle, &messages).await?;
    }
    Ok(())
}
