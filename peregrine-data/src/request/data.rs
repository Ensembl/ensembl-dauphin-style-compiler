use crate::metric::datastreammetric::DatastreamMetricKey;
use crate::metric::datastreammetric::DatastreamMetricValue;
use crate::metric::metricreporter::MetricCollector;
use anyhow::{ anyhow as err };
use serde::Serializer;
use serde::ser::SerializeSeq;
use std::any::Any;
use std::collections::HashMap;
use super::channel::{ Channel, PacketPriority };
use super::request::NewRequestType;
use crate::Region;
use crate::util::cbor::{ cbor_map, cbor_map_iter, cbor_string, cbor_bytes };
use super::backoff::Backoff;
use super::request::{ ResponseType, ResponseBuilderType };
use serde_cbor::Value as CborValue;
use super::manager::RequestManager;
use crate::util::message::DataMessage;

pub struct DataResponse {
    data: HashMap<String,Vec<u8>>,
}

pub(super) async fn do_data_request(channel: &Channel, name: &str, region: &Region, manager: &RequestManager, priority: &PacketPriority, metrics: &MetricCollector) -> Result<Box<DataResponse>,DataMessage> {
    let request = DataCommandRequest::new(channel,name,region);
    request.execute(manager,priority,metrics).await
}

#[derive(Clone)]
pub(super) struct DataCommandRequest {
    channel: Channel,
    name: String,
    region: Region
}

impl DataCommandRequest {
    fn new(channel: &Channel, name: &str, region: &Region) -> DataCommandRequest {
        DataCommandRequest {
            channel: channel.clone(),
            name: name.to_string(),
            region: region.clone()
        }
    }

    fn account(&self, response: &DataResponse, metrics: &MetricCollector, priority: &PacketPriority) {
        for (name,data) in &response.data {
            let key = DatastreamMetricKey::new(&self.name,name,self.region.scale().get_index(),priority.clone());
            let mut value = DatastreamMetricValue::empty();
            value.num_events += 1;
            value.total_size += data.len();
            metrics.add_datastream(&key,&value);
        }
    }

    async fn execute(self, manager: &RequestManager, priority: &PacketPriority, metrics: &MetricCollector) -> Result<Box<DataResponse>,DataMessage> {
        let mut backoff = Backoff::new(manager,&self.channel,priority);
        let mut out = backoff.backoff_new::<DataResponse>(NewRequestType::new_data(self.clone())).await?
                .map_err(|e| DataMessage::DataUnavailable(self.channel.clone(),Box::new(e)));
        if let Ok(response) = &mut out {
            self.account(&response,metrics,priority);
        }
        out
    }
}

impl serde::Serialize for DataCommandRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut seq = serializer.serialize_seq(Some(3))?;
        seq.serialize_element(&self.channel)?;
        seq.serialize_element(&self.name)?;
        seq.serialize_element(&self.region)?;
        seq.end()
    }
}

impl ResponseType for DataResponse {
    fn as_any(&self) -> &dyn Any { self }
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
}

impl DataResponse {
    pub fn get(&self, name: &str) -> anyhow::Result<&Vec<u8>> {
        self.data.get(name).ok_or_else(|| err!("no such data {}",name))
    }
}

pub struct DataResponseBuilderType();

impl ResponseBuilderType for DataResponseBuilderType {
    fn deserialize(&self, value: &CborValue) -> anyhow::Result<Box<dyn ResponseType>> {
        let mut data_response = DataResponse {
            data: HashMap::new()
        };
        let values = cbor_map(value,&["data"])?;
        for (key,value) in cbor_map_iter(&values[0])? {
            // TODO clean, copy-free path
            data_response.data.insert(cbor_string(key)?,cbor_bytes(value)?.clone());
        }
        Ok(Box::new(data_response))
    }
}
