use std::{collections::HashMap, sync::{Arc, Mutex}, rc::Rc};
use peregrine_toolkit::{lock, error::Error};
use crate::{Stick, StickId, metric::{datastreammetric::PacketDatastreamMetricBuilder, metricreporter::MetricCollector}, request::minirequests::{datareq::DataRequest, datares::{DataResponse}, jumpreq::JumpReq, jumpres::{JumpLocation, JumpRes}, programreq::ProgramReq, stickreq::StickReq, stickres::StickRes, expandreq::ExpandReq}, PacketPriority, BackendNamespace, shapeload::programname::ProgramName};
use super::{minirequest::{MiniRequest}, manager::{RequestManager}, miniresponse::{MiniResponseAttempt, MiniResponseError}};

#[derive(Clone)]
pub struct Backend {
    manager: RequestManager,
    name: BackendNamespace,
    metrics: MetricCollector
}

impl Backend {
    pub(crate) fn new(manager: &RequestManager, channel: &BackendNamespace, metrics: &MetricCollector) -> Backend {
        Backend {
            manager: manager.clone(),
            name: channel.clone(),
            metrics: metrics.clone()
        }
    }

    pub fn backend_namespace(&self) -> &BackendNamespace { &self.name }

    async fn submit<F,T>(&self, priority: &PacketPriority, request: MiniRequest, cb: F) -> Result<T,Error>
            where F: Fn(MiniResponseAttempt) -> Result<T,MiniResponseError> {
        self.manager.submit(&self.name,priority,&Rc::new(request), |v| {
            cb(v)
        }).await
    }

    async fn submit_hi<F,T>(&self, request: MiniRequest, cb: F) -> Result<T,Error>
            where F: Fn(MiniResponseAttempt) -> Result<T,MiniResponseError> {
        self.submit(&PacketPriority::RealTime,request,cb).await
    }

    pub async fn data(&self, data_request: &DataRequest, priority: &PacketPriority) -> Result<DataResponse,Error> {
        let request = MiniRequest::Data(data_request.clone());
        let account_builder = PacketDatastreamMetricBuilder::new(&self.metrics,data_request.name(),priority,data_request.region());
        let r = self.submit(priority,request, |d| {
            Ok(DataResponse::new(d.into_variety().into_data()?,priority.clone()))
        }).await?;
        r.account(&account_builder);
        Ok(r)
    }

    pub async fn stick(&self, id: &StickId) -> Result<Option<Stick>,Error> {
        let req = StickReq::new(&id);
        let r = self.submit_hi(req, |d| { d.into_variety().into_stick() }).await?;
        match r {
            StickRes::Stick(s) => Ok(Some(s)),
            StickRes::Unknown(_) => Ok(None)
        }
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

    pub async fn expand(&self, name: &str, step: &str) -> Result<(),Error> {
        let req = ExpandReq::new(name,step);
        self.submit_hi(req, |d| d.into_variety().into_expand()).await?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct AllBackends {
    manager: RequestManager,
    metrics: MetricCollector,
    backends: Arc<Mutex<HashMap<BackendNamespace,Backend>>>
}

impl AllBackends {
    pub fn new(manager: &RequestManager, metrics: &MetricCollector) -> AllBackends {
        AllBackends {
            manager: manager.clone(),
            metrics: metrics.clone(),
            backends: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    pub fn backend(&self, channel: &BackendNamespace) -> Result<Backend,Error> {
        let mut backends = lock!(self.backends);
        if !backends.contains_key(channel) {
            backends.insert(channel.clone(), Backend::new(&self.manager,channel,&self.metrics));
        }
        Ok(backends.get(channel).unwrap().clone())
    }

    pub fn all(&self) -> Vec<Backend> {
        lock!(self.backends).values().cloned().collect()
    }
}
