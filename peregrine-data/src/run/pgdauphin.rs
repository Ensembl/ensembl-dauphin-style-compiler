use anyhow::{ self };
use peregrine_toolkit::error::Error;
use peregrine_toolkit::{lock, log};
use std::any::Any;
use std::collections::{HashMap};
use std::sync::{ Arc, Mutex };
use crate::core::channel::channelregistry::ChannelRegistry;
use crate::core::program::programspec::ProgramModel;
use crate::{MaxiResponse, BackendNamespace, AccessorResolver, LoadMode, AllBackends, CountingPromise};
use crate::api::MessageSender;
use crate::core::program::programbundle::SuppliedBundle;
use peregrine_dauphin_queue::{ PgDauphinQueue, PgDauphinLoadTaskSpec, PgEardoLoadTaskSpec, PgEardoRunTaskSpec };
use crate::shapeload::programname::{ProgramName};
use eard_interp::ObjectFile;

pub(crate) struct PgEardoTaskSpec {
    pub(crate) program: eard_interp::ProgramName,
    pub(crate) track_base: BackendNamespace,
    pub(crate) payloads: Option<HashMap<String,Box<dyn Any>>>
}

struct PgDauphinData {
    pdq: PgDauphinQueue,
    programs_present: HashMap<eard_interp::ProgramName,BackendNamespace>,
    programs: HashMap<ProgramName,Option<ProgramModel>>,
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
            programs_present: HashMap::new(),
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

    fn register_eardo(&self, backend_namespace: &BackendNamespace, data: &[u8]) -> Result<(),Error> {
        let eardo = ObjectFile::decode(data.to_vec()).map_err(|e| 
             Error::operr(&format!("cannot read file: {}",e))
        )?;
        for name in eardo.list_programs() {
            log!("registering {:?}",name);
            lock!(self.0).programs_present.insert(name,backend_namespace.clone());
        }
        Ok(())
    }

    async fn add_eardo(&self, backend_namespace: &BackendNamespace, name: &str, data: &[u8]) -> Result<(),Error> {
        let obj = lock!(self.0);
        let pdq = obj.pdq.clone();
        pdq.load_eardo(PgEardoLoadTaskSpec {
            bundle_name: name.to_string(),
            data: data.to_vec()
        }).await?;
        drop(obj);
        self.register_eardo(backend_namespace,data)?;
        Ok(())
    }

    fn register(&self, program: &ProgramModel) {
        lock!(self.0).programs.insert(program.name().clone(),Some(program.clone()));
    }

    pub fn is_present(&self, program_name: &ProgramName) -> bool {
        if lock!(self.0).programs_present.contains_key(program_name.to_eard()) { return true; }
        lock!(self.0).programs.get(program_name).and_then(|x| x.as_ref()).is_some()
    }

    pub fn mark_missing(&self, program_name: &ProgramName) {
        let mut data = lock!(self.0);
        data.programs.insert(program_name.clone(),None);
    }

    async fn get_program(&self, program_name: &ProgramName) -> Result<ProgramModel,Error> {
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
        match self.get_program(program_name).await {
            Ok(program) => Ok(program),
            Err(_) => Ok(ProgramModel::empty(program_name))
        }
    }

    pub(crate) async fn run_eardo(&self, registry: &ChannelRegistry, spec: PgEardoTaskSpec, mode: &LoadMode) -> Result<(),Error> {
        let program_backend = lock!(self.0).programs_present.get(&spec.program).cloned().ok_or_else(||
            Error::operr(&format!("cannot find program {:?}",spec.program))
        )?;
        let mut payloads = spec.payloads.unwrap_or_else(|| HashMap::new());
        payloads.insert("channel-resolver".to_string(),Box::new(AccessorResolver::new(registry,&program_backend,&spec.track_base)));
        let pdq = lock!(self.0).pdq.clone();
        pdq.run_eardo(PgEardoRunTaskSpec {
            prio: if mode.high_priority() { 2 } else { 9 },
            name: spec.program,
            payloads
        }).await
    }
}

async fn add_eardo(pgd: &PgDauphin, backend_namespace: &BackendNamespace, name: &str, data: &[u8], messages: &MessageSender) -> Result<(),Error> {
    //let specs = bundle.specs().to_program_models()?;
    match pgd.add_eardo(backend_namespace,name,data).await {
        Ok(_) => {},
        Err(e) => {
            messages.send(e);
        }
    }
    Ok(())
}

pub(crate) async fn add_programs_from_response(pgd: &PgDauphin, channel: &BackendNamespace, response: &MaxiResponse, messages: &MessageSender) -> Result<(),Error> {
    for (name,data) in response.eardos() {
        add_eardo(&pgd,channel,&name,data,&messages).await?;
    }
    Ok(())
}
