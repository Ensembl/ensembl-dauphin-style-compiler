use std::{collections::HashMap, sync::{Arc, Mutex}};
use peregrine_toolkit::lock;

use crate::{Channel, DataMessage, PacketPriority, ProgramName, Region, RequestManager, Stick, StickId, index::stickauthority::Authority, metric::metricreporter::MetricCollector};
use super::{bootstrap::{BootstrapCommandResponse, do_bootstrap}, data::{DataResponse, do_data_request}, jump::{JumpLocation, do_jump_request}, program::do_load_program, stick::do_stick_request, authority::do_stick_authority};

#[derive(Clone)]
pub struct Backend {
    manager: RequestManager,
    channel: Channel,
    metrics: MetricCollector
}

impl Backend {
    pub(crate) fn new(manager: &RequestManager, channel: &Channel, metrics: &MetricCollector) -> Backend {
        Backend {
            manager: manager.clone(),
            channel: channel.clone(),
            metrics: metrics.clone()
        }
    }

    pub async fn data(&self, name: &str, region: &Region, priority: &PacketPriority) -> Result<Box<DataResponse>,DataMessage> {
        do_data_request(&self.channel,name,region,&self.manager,priority,&self.metrics).await
    }

    pub async fn stick(&self, id: &StickId) -> anyhow::Result<Stick> {
        do_stick_request(self.manager.clone(),self.channel.clone(),id.clone()).await
    }

    pub async fn authority(&self) -> Result<Authority,DataMessage> {
        do_stick_authority(self.manager.clone(),self.channel.clone()).await
    }

    pub async fn jump(&self, location: &str) -> anyhow::Result<Option<JumpLocation>> {
        do_jump_request(self.manager.clone(),self.channel.clone(),location.clone()).await
    }

    pub async fn bootstrap(&self) -> Result<Box<BootstrapCommandResponse>,DataMessage> {
        do_bootstrap(&self.manager,&self.channel).await
    }

    pub async fn program(&self, program_name: &ProgramName) -> Result<(),DataMessage> {
        do_load_program(&self.manager,program_name.clone()).await
    }
}

#[derive(Clone)]
pub struct AllBackends {
    manager: RequestManager,
    metrics: MetricCollector,
    backends: Arc<Mutex<HashMap<Channel,Backend>>>
}

impl AllBackends {
    pub fn new(manager: &RequestManager, metrics: &MetricCollector) -> AllBackends {
        AllBackends {
            manager: manager.clone(),
            metrics: metrics.clone(),
            backends: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    pub fn backend(&self, channel: &Channel) -> Backend {
        let mut backends = lock!(self.backends);
        if !backends.contains_key(channel) {
            backends.insert(channel.clone(), Backend::new(&self.manager,channel,&self.metrics));
        }
        backends.get(channel).unwrap().clone()
    }
}
