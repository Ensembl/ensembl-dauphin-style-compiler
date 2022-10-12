use std::{collections::HashMap, sync::{Arc, Mutex}, rc::Rc};
use peregrine_toolkit::{lock, error::Error};
use crate::{DataMessage, ProgramName, Stick, StickId, api::MessageSender, index::stickauthority::Authority, metric::{datastreammetric::PacketDatastreamMetricBuilder, metricreporter::MetricCollector}, request::minirequests::{authorityreq::AuthorityReq, datareq::DataRequest, datares::{DataResponse}, jumpreq::JumpReq, jumpres::{JumpLocation, JumpRes}, programreq::ProgramReq, stickreq::StickReq, stickres::StickRes}, PacketPriority, BackendNamespace};
use super::{request::{MiniRequest}, manager::{RequestManager}, response::MiniResponseAttempt};

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

    pub fn backend_namespace(&self) -> &BackendNamespace { &self.name }

    async fn submit<F,T>(&self, priority: &PacketPriority, request: MiniRequest, cb: F) -> Result<T,Error>
            where F: Fn(MiniResponseAttempt) -> Result<T,String> {
        self.manager.submit(&self.name,priority,&Rc::new(request), |v| {
            cb(v)
        }).await
    }

    async fn submit_hi<F,T>(&self, request: MiniRequest, cb: F) -> Result<T,Error>
            where F: Fn(MiniResponseAttempt) -> Result<T,String> {
        self.submit(&PacketPriority::RealTime,request,cb).await
    }

    pub async fn data(&self, data_request: &DataRequest, priority: &PacketPriority) -> Result<DataResponse,Error> {
        let request = MiniRequest::Data(data_request.clone());
        let account_builder = PacketDatastreamMetricBuilder::new(&self.metrics,data_request.name(),priority,data_request.region());
        let r = self.submit(priority,request, |d| {
            Ok(DataResponse::new(d.into_variety().into_data()?))
        }).await?;
        r.account(&account_builder);
        Ok(r)
    }

    pub async fn stick(&self, id: &StickId) -> Result<Stick,Error> {
        let req = StickReq::new(&id);
        let r = self.submit_hi(req, |d| { d.into_variety().into_stick() }).await?;
        match r {
            StickRes::Stick(s) => Ok(s),
            StickRes::Unknown(_) => {
                Err(Error::operr(&format!("No such stick: {}",id)))
            }
        }
    }

    pub async fn authority(&self) -> Result<Authority,Error> {
        let request = AuthorityReq::new();
        Ok(self.submit_hi(request, |d| d.into_variety().into_authority()).await?.build())
    }

    pub async fn jump(&self, location: &str) -> Result<Option<JumpLocation>,Error> {
        let req = JumpReq::new(&location);
        let r = self.submit_hi(req, |d| d.into_variety().into_jump()).await?;
        Ok(match r {
            JumpRes::Found(x) => Some(x),
            JumpRes::NotFound => None
        })
    }

    pub async fn program(&self, program_name: &ProgramName) -> Result<(),Error> {
        let req = ProgramReq::new(&program_name);
        self.submit_hi(req, |d| d.into_variety().into_program()).await?;
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

    pub fn backend(&self, channel: &BackendNamespace) -> Result<Backend,Error> {
        let mut backends = lock!(self.backends);
        if !backends.contains_key(channel) {
            backends.insert(channel.clone(), Backend::new(&self.manager,channel,&self.metrics,&self.messages));
        }
        Ok(backends.get(channel).unwrap().clone())
    }

    pub fn all(&self) -> Vec<Backend> {
        lock!(self.backends).values().cloned().collect()
    }
}
