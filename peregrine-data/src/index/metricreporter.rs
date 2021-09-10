use commander::cdr_timer;
use commander::cdr_add_timer;
use std::sync::Mutex;
use std::sync::Arc;
use std::collections::{BTreeMap, HashMap};

use peregrine_message::PeregrineMessage;
use crate::{Channel, PacketPriority, PgCommander, PgCommanderTaskSpec, RequestManager, add_task, request::{failure::GeneralFailure, request::RequestType}, DataMessage};
use crate::PeregrineCoreBase;
use serde_derive::{ Serialize };

#[derive(Clone,Serialize)]
#[serde(tag = "type")]
#[cfg_attr(debug_assertions,derive(Debug))]
pub enum MetricReport {
    Datastream(DatastreamMetricReport),
    Error(ErrorMetricReport)
}

#[derive(Clone,Serialize)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct ErrorMetricReport {
    identity: u64,
    text: String,
    major: u64,
    minor: u64
}

#[derive(Clone,Serialize)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct DatastreamMetric {
    pub name: String,
    pub scale: u64,
    pub num_events: u64,
    pub total_size: usize
}

impl DatastreamMetric {
    pub fn empty(name: &str, scale: u64) -> DatastreamMetric {
        DatastreamMetric {
            name: name.to_string(),
            scale,
            num_events: 0,
            total_size: 0
        }
    }
}

#[derive(Clone,Serialize)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct DatastreamMetricReport {
    identity: u64,
    timings: Arc<Vec<DatastreamMetric>>
}

impl DatastreamMetricReport {
    fn new(identity: u64, generator: &mut DatastreamMetricReportGenerator) -> DatastreamMetricReport {
        let timings = generator.timings.drain().map(|(_,v)| v).collect::<Vec<_>>();
        DatastreamMetricReport {
            identity,
            timings: Arc::new(timings)
        }
    }

    fn len(&self) -> usize { self.timings.len() }
}

pub struct DatastreamMetricReportGenerator {
    timings: HashMap<(String,u64),DatastreamMetric>
}

impl DatastreamMetricReportGenerator {
    pub(crate) fn new() -> DatastreamMetricReportGenerator {
        DatastreamMetricReportGenerator {
            timings: HashMap::new()
        }
    }

    pub(crate) fn add(&mut self, metric: &DatastreamMetric) {
        let e = self.timings.entry((metric.name.to_string(),metric.scale))
            .or_insert(DatastreamMetric::empty(&metric.name,metric.scale));
        e.num_events += metric.num_events;
        e.total_size += metric.total_size;
    }
}

impl MetricReport {
    pub fn new_from_error_message(base: &PeregrineCoreBase, message: &(dyn PeregrineMessage + 'static)) -> MetricReport {
        let identity = *base.identity.lock().unwrap();
        MetricReport::Error(ErrorMetricReport {
            identity,
            text: message.to_string(),
            major: message.code().0,
            minor: message.code().1
        })
    }

    async fn send_task(&self, mut manager: RequestManager, channel: Channel) {
        // We don't care about errors here: avoid loops and spew
        manager.execute(channel,PacketPriority::Batch,Box::new(self.clone())).await.ok();
    }

    pub(crate) fn send(&self, commander: &PgCommander, manager: &mut RequestManager, channel: &Channel) {
        let self2 = self.clone();
        let manager = manager.clone();
        let channel = channel.clone();
        add_task(commander,PgCommanderTaskSpec {
            name: "message".to_string(),
            prio: 11,
            timeout: None,
            slot: None,
            task: Box::pin(async move { 
                self2.send_task(manager,channel).await;
                Ok(())
            }),
            stats: false
        });
    }
}

impl RequestType for MetricReport {
    fn type_index(&self) -> u8 { 6 }

    fn serialize(&self, channel: &Channel) -> Result<serde_cbor::Value,DataMessage> {
        serde_cbor::value::to_value(self).map_err(|e| DataMessage::PacketError(channel.clone(),e.to_string()))
    }

    fn to_failure(&self) -> Box<dyn crate::request::request::ResponseType> {
        Box::new(GeneralFailure::new("metric reporting failed"))
    }
}


struct MetricCollectorData {
    datastream: DatastreamMetricReportGenerator,
    manager: Option<RequestManager>,
    channel: Option<Channel>,
    identity: u64
}

impl MetricCollectorData {
    fn new() -> MetricCollectorData {
        MetricCollectorData {
            datastream: DatastreamMetricReportGenerator::new(),
            manager: None,
            channel: None,
            identity: 0
        }
    }

    pub fn bootstrap(&mut self, channel: &Channel, identity: u64, manager: &RequestManager) {
        self.channel = Some(channel.clone());
        self.identity = identity;
        self.manager = Some(manager.clone());
    }

    async fn send(&mut self) {
        let report = DatastreamMetricReport::new(self.identity,&mut self.datastream);
        if report.len() > 0 {
            if let (Some(manager),Some(channel)) = (&mut self.manager,&self.channel) {
                manager.execute(channel.clone(),PacketPriority::Batch,Box::new(MetricReport::Datastream(report))).await.ok();   
            }
        }
    }
}

#[derive(Clone)]
pub struct MetricCollector(Arc<Mutex<MetricCollectorData>>);

impl MetricCollector {
    async fn run(&self) {
        loop {
            self.0.lock().unwrap().send().await;
            cdr_timer(60000.).await;
        }
    }

    pub fn new(commander: &PgCommander) -> MetricCollector {
        let out = MetricCollector(Arc::new(Mutex::new(MetricCollectorData::new())));
        let out2 = out.clone();
        add_task(commander,PgCommanderTaskSpec {
            name: "metric-sender".to_string(),
            prio: 11,
            timeout: None,
            slot: None,
            task: Box::pin(async move { 
                out2.run().await;
                Ok(())
            }),
            stats: false
        });
        out
    }

    pub fn bootstrap(&self, channel: &Channel, identity: u64, manager: &RequestManager) {
        self.0.lock().unwrap().bootstrap(channel,identity,manager);
    }

    pub fn add_datastream(&self, metric: &DatastreamMetric) {
        self.0.lock().unwrap().datastream.add(metric);
    }
}
