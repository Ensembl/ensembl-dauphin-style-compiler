use std::{collections::HashMap, sync::{Arc, Mutex}};
use peregrine_toolkit::lock;

use crate::{DataMessage, ProgramName, Region, RequestManager, Stick, StickId, core::channel::{Channel, PacketPriority}, index::stickauthority::Authority, metric::{datastreammetric::PacketDatastreamMetricBuilder, metricreporter::MetricCollector}, request::messages::{authorityreq::AuthorityReq, bootstrapreq::BootstrapReq, bootstrapres::BootRes, datareq::DataReq, datares::DataRes, jumpreq::JumpReq, jumpres::{JumpLocation, JumpRes}, programreq::ProgramReq, stickreq::StickReq}};

use super::request::RequestType;

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

    pub async fn data(&self, name: &str, region: &Region, priority: &PacketPriority) -> Result<DataRes,DataMessage> {
        let request = DataReq::new(&self.channel,name,region);
        let account_builder = PacketDatastreamMetricBuilder::new(&self.metrics,name,priority,region);
        let r = self.manager.submit(&self.channel,priority,RequestType::new(request), |v| {
            v.into_data()
        }).await?;
        r.account(&account_builder);
        Ok(r)
    }

    pub async fn stick(&self, id: &StickId) -> anyhow::Result<Stick> {
        let req = StickReq::new(&id);
        let r = self.manager.submit(&self.channel, &PacketPriority::RealTime, RequestType::new(req), |v| {
            v.into_stick()
        }).await?;
        Ok(r.stick())
    }

    pub async fn authority(&self) -> Result<Authority,DataMessage> {
        let request = AuthorityReq::new();
        Ok(self.manager.submit(&self.channel,&PacketPriority::RealTime, RequestType::new(request), |v| {
            v.into_authority()
        }).await?.build())
    }

    pub async fn jump(&self, location: &str) -> anyhow::Result<Option<JumpLocation>> {
        let req = JumpReq::new(&location);
        let r = self.manager.submit(&self.channel,&PacketPriority::RealTime,RequestType::new(req), |v| {
            v.into_jump()
        }).await?;
        Ok(match r {
            JumpRes::Found(x) => Some(x),
            JumpRes::NotFound => None
        })
    }

    pub async fn bootstrap(&self) -> Result<BootRes,DataMessage> {
        let request = BootstrapReq::new();
        self.manager.submit(&self.channel,&PacketPriority::RealTime,RequestType::new(request), |v| {
            v.into_bootstrap()
        }).await    
    }

    pub async fn program(&self, program_name: &ProgramName) -> Result<(),DataMessage> {
        let req = ProgramReq::new(&program_name);
        self.manager.submit(&program_name.0,&PacketPriority::RealTime,RequestType::new(req), |v| {
            v.into_program()
        }).await?;
        Ok(())
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
