use std::{collections::HashMap, sync::{Arc, Mutex}};
use peregrine_toolkit::lock;

use crate::{DataMessage, ProgramName, Stick, StickId, api::MessageSender, index::stickauthority::Authority, metric::{datastreammetric::PacketDatastreamMetricBuilder, metricreporter::MetricCollector}, request::messages::{authorityreq::AuthorityReq, datareq::DataRequest, datares::DataRes, jumpreq::JumpReq, jumpres::{JumpLocation, JumpRes}, programreq::ProgramReq, stickreq::StickReq}, PacketPriority, BackendNamespace};

use super::{request::{BackendRequest}, manager::{RequestManager}, response::BackendResponse};

#[derive(Clone)]
pub struct Backend {
    manager: RequestManager,
    messages: MessageSender,
    name: BackendNamespace,
    metrics: MetricCollector
}

impl Backend {
    pub(crate) fn new(manager: &RequestManager, channel: &BackendNamespace, metrics: &MetricCollector, messages: &MessageSender) -> Backend {
        Backend {
            manager: manager.clone(),
            messages: messages.clone(),
            name: channel.clone(),
            metrics: metrics.clone()
        }
    }

    async fn submit<F,T>(&self, priority: &PacketPriority, request: &BackendRequest, cb: F) -> Result<T,DataMessage>
            where F: Fn(BackendResponse) -> Result<T,String> {
        self.manager.submit(&self.name,priority,&request, |v| {
            cb(v)
        }).await
    }

    async fn submit_hi<F,T>(&self, request: &BackendRequest, cb: F) -> Result<T,DataMessage>
            where F: Fn(BackendResponse) -> Result<T,String> {
        self.submit(&PacketPriority::RealTime,request,cb).await
    }

    pub async fn data(&self, data_request: &DataRequest, priority: &PacketPriority) -> Result<DataRes,DataMessage> {
        let request = BackendRequest::Data(data_request.clone());
        let account_builder = PacketDatastreamMetricBuilder::new(&self.metrics,data_request.name(),priority,data_request.region());
        let r = self.submit(priority,&request, |v| { v.into_data() }).await?;
        r.account(&account_builder);
        Ok(r)
    }

    pub async fn stick(&self, id: &StickId) -> Result<Stick,DataMessage> {
        let req = StickReq::new(&id);
        let r = self.submit_hi(&req, |v| { v.into_stick() }).await?;
        match r.stick() {
            Ok(s) => Ok(s),
            Err(_e) => {
                self.messages.send(DataMessage::NoSuchStick(id.clone()));
                Err(DataMessage::NoSuchStick(id.clone()))
            }
        }
    }

    pub async fn authority(&self) -> Result<Authority,DataMessage> {
        let request = AuthorityReq::new();
        Ok(self.submit_hi(&request, |v| v.into_authority()).await?.build())
    }

    pub async fn jump(&self, location: &str) -> anyhow::Result<Option<JumpLocation>> {
        let req = JumpReq::new(&location);
        let r = self.submit_hi(&req, |v| v.into_jump()).await?;
        Ok(match r {
            JumpRes::Found(x) => Some(x),
            JumpRes::NotFound => None
        })
    }

    pub async fn program(&self, program_name: &ProgramName) -> Result<(),DataMessage> {
        let req = ProgramReq::new(&program_name);
        self.submit_hi(&req, |v| v.into_program()).await?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct AllBackends {
    manager: RequestManager,
    messages: MessageSender,
    metrics: MetricCollector,
    backends: Arc<Mutex<HashMap<BackendNamespace,Backend>>>
}

impl AllBackends {
    pub fn new(manager: &RequestManager, metrics: &MetricCollector, messages: &MessageSender) -> AllBackends {
        AllBackends {
            manager: manager.clone(),
            metrics: metrics.clone(),
            messages: messages.clone(),
            backends: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    pub fn backend(&self, channel: &BackendNamespace) -> Backend {
        let mut backends = lock!(self.backends);
        if !backends.contains_key(channel) {
            backends.insert(channel.clone(), Backend::new(&self.manager,channel,&self.metrics,&self.messages));
        }
        backends.get(channel).unwrap().clone()
    }
}
