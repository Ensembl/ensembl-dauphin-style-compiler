use std::{collections::HashMap, rc::Rc, sync::{Arc, Mutex}};
use peregrine_toolkit::{ lock, error::Error };
use crate::{ChannelIntegration, BackendNamespace, DataMessage, PeregrineCoreBase, CountingPromise };
use super::{channelboot::{ChannelBoot }, wrappedchannelsender::WrappedChannelSender };

pub struct ChannelRegistryBuilder {
    integrations: Vec<Rc<dyn ChannelIntegration>>,
    boot: ChannelBoot
}

impl ChannelRegistryBuilder {
    pub(crate) fn new(booted: &CountingPromise) -> ChannelRegistryBuilder {
        ChannelRegistryBuilder {
            integrations: vec![],
            boot: ChannelBoot::new(booted)
        }
    }

    pub(crate) fn add(&mut self, integration: Rc<dyn ChannelIntegration>) {
        self.integrations.push(integration.clone());
    }

    pub(crate) fn build(self) -> ChannelRegistry {
        ChannelRegistry::new(self.integrations,self.boot)
    }
}

/* Invariant: value in spec_to_channel will have key in channels. */
#[derive(Clone)]
pub struct ChannelRegistry {
    integrations: Arc<Vec<Rc<dyn ChannelIntegration>>>,
    spec_to_channel: Arc<Mutex<HashMap<String,BackendNamespace>>>,
    channels: Arc<Mutex<HashMap<BackendNamespace,WrappedChannelSender>>>,
    boot: ChannelBoot
}

impl ChannelRegistry {
    fn new(integrations: Vec<Rc<dyn ChannelIntegration>>, boot: ChannelBoot) -> ChannelRegistry {
        ChannelRegistry {
            integrations: Arc::new(integrations),
            channels: Arc::new(Mutex::new(HashMap::new())),
            spec_to_channel: Arc::new(Mutex::new(HashMap::new())),
            boot
        }
    }

    pub(crate) fn run_boot_loop(&self, base: &PeregrineCoreBase) {
        self.boot.run_boot_loop(base);
    }

    pub(super) fn register_channel(&self, backend_namespace: &BackendNamespace, sender:&WrappedChannelSender) {
        lock!(self.channels).insert(backend_namespace.clone(),sender.clone());
    }

    async fn lookup_unknown_spec(&self, access: &str) -> Result<BackendNamespace,Error> {
        for itn in self.integrations.iter() {
            if let Some((sender,backend_namespace)) = itn.make_channel(access) {
                let backend_namespace = BackendNamespace::or_missing(&backend_namespace);
                let sender = WrappedChannelSender::new(sender);
                let backend_namespace = self.boot.boot(&backend_namespace,&sender).await?;
                self.register_channel(&backend_namespace,&sender); // should already have been done by boot, but let's do it here to keep invariant explicit
                lock!(self.channels).insert(backend_namespace.clone(),sender.clone());
                lock!(self.spec_to_channel).insert(access.to_string(),backend_namespace.clone());
                return Ok(backend_namespace)
            }
        }
        Err(Error::operr(&format!("unclaimed {}",access)))
    }

    pub(crate) async fn spec_to_name(&self, access: &str) -> Result<BackendNamespace,Error> {
        let missing = !lock!(self.spec_to_channel).contains_key(access);
        if missing {
            self.lookup_unknown_spec(access).await?; // is idempotent, so no race
        }
        lock!(self.spec_to_channel).get(access).cloned().ok_or_else(|| Error::operr(&format!("unclaimed {}",access)))
    }

    pub async fn add_backend(&self, access: &str) -> Result<(),Error> {
        self.spec_to_name(access).await.map(|_| ())
    }

    pub async fn booted(&self) -> Result<(),DataMessage> {
        self.boot.ready().await
    }

    pub(crate) fn name_to_sender(&self, name: &BackendNamespace) -> Result<WrappedChannelSender,Error> {
        let channels = lock!(self.channels);
        Ok(channels.get(name).ok_or_else(|| Error::operr(&format!("No such backend namespace: {}",name)))?.clone())
    }

    pub(crate) fn all(&self) -> Vec<BackendNamespace> {
        lock!(self.channels).keys().cloned().collect()
    }
}
