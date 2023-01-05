use anyhow::{ self };
use peregrine_toolkit::error::Error;
use peregrine_toolkit::{lock};
use std::any::Any;
use std::collections::HashMap;
use std::sync::{ Arc, Mutex };
use crate::core::channel::channelregistry::ChannelRegistry;
use crate::core::program::programspec::ProgramModel;
use crate::request::tracks::trackmodel::TrackMapping;
use crate::{MaxiResponse, BackendNamespace, AccessorResolver, LoadMode, AllBackends, CountingPromise};
use crate::api::MessageSender;
use crate::core::program::programbundle::SuppliedBundle;
use peregrine_dauphin_queue::{ PgDauphinQueue, PgDauphinLoadTaskSpec, PgDauphinRunTaskSpec, PgEardoLoadTaskSpec, PgEardoRunTaskSpec };
use crate::shapeload::programname::{ProgramName};

pub(crate) struct PgDauphinTaskSpec {
    pub(crate) program: ProgramModel,
    pub(crate) track_base: BackendNamespace,
    pub(crate) mapping: TrackMapping,
    pub(crate) payloads: Option<HashMap<String,Box<dyn Any>>>
}

pub(crate) struct PgEardoTaskSpec {
    pub(crate) program: eard_interp::ProgramName,
    pub(crate) track_base: BackendNamespace,
    pub(crate) mapping: TrackMapping,
    pub(crate) payloads: Option<HashMap<String,Box<dyn Any>>>
}

#[derive(Clone)]
struct Program {
    backend_namespace: BackendNamespace,
    program: ProgramModel,
    bundle_name: String,
    in_bundle_name: String
}

impl Program {
    fn new(backend_namespace: &BackendNamespace, program: &ProgramModel, bundle_name: &str, in_bundle_name: &str) -> Program {
        Program {
            backend_namespace: backend_namespace.clone(),
            bundle_name: bundle_name.to_string(),
            program: program.clone(),
            in_bundle_name: in_bundle_name.to_string()
        }
    }
}

struct PgDauphinData {
    pdq: PgDauphinQueue,
    programs: HashMap<ProgramName,Option<Program>>,
    all_backends: Option<AllBackends>,
    channel_registry: ChannelRegistry
}

#[derive(Clone)]
pub struct PgDauphin(Arc<Mutex<PgDauphinData>>,CountingPromise);

impl PgDauphin {
    pub fn new(pdq: &PgDauphinQueue, channel_registry: &ChannelRegistry, booted: &CountingPromise) -> anyhow::Result<PgDauphin> {
        Ok(PgDauphin(Arc::new(Mutex::new(PgDauphinData {
            pdq: pdq.clone(),
            programs: HashMap::new(),
            all_backends: None,
            channel_registry: channel_registry.clone()
        })),booted.clone()))
    }

    async fn load_program(&self, program_name: &ProgramName) -> Result<(),Error> {
        let obj = lock!(self.0);
        let all_backends = obj.all_backends.clone();
        let channel_registry = obj.channel_registry.clone();
        let programs = obj.programs.clone();
        drop(obj);
        for backend_namespace in &channel_registry.all() {
            if let Some(all_backends) = &all_backends {
                let backend = all_backends.backend(backend_namespace)?;
                backend.program(program_name).await?;
                if let Some(Some(_)) = programs.get(program_name) { break; }
            }
        }
        Ok(())
    }

    pub(crate) fn set_all_backends(&self, all_backends: &AllBackends) {
        lock!(self.0).all_backends = Some(all_backends.clone());
    }

    fn binary_name(&self, channel: &BackendNamespace, name_of_bundle: &str) -> String {
        let channel_name = channel.to_string();
        format!("{}-{}-{}",channel_name.len(),channel_name,name_of_bundle)
    }

    async fn add_binary(&self, channel: &BackendNamespace, name_of_bundle: &str, cbor: &[u8]) -> Result<(),Error> {
        let binary_name = self.binary_name(channel,name_of_bundle);
        let obj = lock!(self.0);
        let pdq = obj.pdq.clone();
        drop(obj);
        pdq.load(PgDauphinLoadTaskSpec {
            bundle_name: binary_name.to_string(),
            data: cbor.to_vec()
        }).await
    }

    async fn add_eardo(&self, name: &str, data: &[u8]) -> Result<(),Error> {
        let obj = lock!(self.0);
        let pdq = obj.pdq.clone();
        pdq.load_eardo(PgEardoLoadTaskSpec {
            bundle_name: name.to_string(),
            data: data.to_vec()
        }).await
    }

    fn register(&self, backend_namespace: &BackendNamespace, program: &ProgramModel, name_of_bundle: &str) {
        let binary_name = self.binary_name(backend_namespace,name_of_bundle);
        let internal_name = Program::new(backend_namespace,program,&binary_name,program.in_bundle_name());
        lock!(self.0).programs.insert(program.name().clone(),Some(internal_name));
    }

    pub fn is_present(&self, program_name: &ProgramName) -> bool {
        lock!(self.0).programs.get(program_name).and_then(|x| x.as_ref()).is_some()
    }

    pub fn mark_missing(&self, program_name: &ProgramName) {
        let mut data = lock!(self.0);
        data.programs.insert(program_name.clone(),None);
    }

    async fn get_program(&self, program_name: &ProgramName) -> Result<Program,Error> {
        let program_name = program_name.clone();
        if !self.is_present(&program_name) {
            self.load_program(&program_name).await?;
        }
        let data = lock!(self.0);
        if let Some(Some(program)) = data.programs.get(&program_name) {
            Ok(program.clone())
        } else {
            Err(Error::operr(&format!("failed channel/program {:?}",program_name)))
        }
    }

    pub(crate) async fn get_program_model(&self, program_name: &ProgramName) -> Result<ProgramModel,Error> {
        let program = self.get_program(program_name).await?;
        Ok(program.program)
    }

    pub(crate) async fn run_program(&self, registry: &ChannelRegistry, spec: PgDauphinTaskSpec, mode: &LoadMode) -> Result<(),Error> {
        let program = self.get_program(&spec.program.name()).await?;
        let mut payloads = spec.payloads.unwrap_or_else(|| HashMap::new());
        payloads.insert("channel-resolver".to_string(),Box::new(AccessorResolver::new(registry,&program.backend_namespace,&spec.track_base)));
        payloads.insert("program-spec".to_string(),Box::new(spec.program.clone()) as Box::<dyn Any>);
        payloads.insert("track-mapping".to_string(),Box::new(spec.mapping.clone()) as Box::<dyn Any>);
        let pdq = lock!(self.0).pdq.clone();
        pdq.run(PgDauphinRunTaskSpec {
            prio: if mode.high_priority() { 2 } else { 9 },
            slot: None,
            timeout: None,
            bundle_name: program.bundle_name.to_string(), 
            in_bundle_name: program.in_bundle_name.to_string(),
            payloads
        }).await
    }

    pub(crate) async fn run_eardo(&self, registry: &ChannelRegistry, spec: PgEardoTaskSpec, mode: &LoadMode) -> Result<(),Error> {
        let pdq = lock!(self.0).pdq.clone();
        pdq.run_eardo(PgEardoRunTaskSpec {
            prio: if mode.high_priority() { 2 } else { 9 },
            name: spec.program,
        }).await
    }
}

async fn add_bundle(pgd: &PgDauphin, channel: &BackendNamespace, bundle: &SuppliedBundle, messages: &MessageSender) -> Result<(),Error> {
    let specs = bundle.specs().to_program_models()?;
    match pgd.add_binary(&channel,bundle.bundle_name(),bundle.code()).await {
        Ok(_) => {
            for spec in specs {
                pgd.register(channel,&spec,bundle.bundle_name());
            }
        },
        Err(e) => {
            messages.send(e);
            for spec in specs {
                pgd.mark_missing(spec.name());
            }
        }
    }
    Ok(())
}

async fn add_eardo(pgd: &PgDauphin, name: &str, data: &[u8], messages: &MessageSender) -> Result<(),Error> {
    //let specs = bundle.specs().to_program_models()?;
    match pgd.add_eardo(name,data).await {
        Ok(_) => {
         //   for spec in specs {
         //       pgd.register(channel,&spec,bundle.bundle_name());
         //   }
        },
        Err(e) => {
            messages.send(e);
         //   for spec in specs {
           //     pgd.mark_missing(spec.name());
         //   }
        }
    }
    Ok(())
}

pub(crate) async fn add_programs_from_response(pgd: &PgDauphin, channel: &BackendNamespace, response: &MaxiResponse, messages: &MessageSender) -> Result<(),Error> {
    for bundle in response.programs().clone().iter() {
        add_bundle(&pgd,&channel, bundle, &messages).await?;
    }
    for (name,data) in response.eardos() {
        add_eardo(&pgd,&name,data,&messages).await?;
    }
    Ok(())
}
