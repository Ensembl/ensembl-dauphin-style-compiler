use crate::metric::datastreammetric::DatastreamMetricKey;
use crate::metric::datastreammetric::DatastreamMetricValue;
use crate::metric::metricreporter::MetricCollector;
use anyhow::{ anyhow as err };
use std::any::Any;
use std::collections::HashMap;
use super::channel::{ Channel, PacketPriority };
use crate::Region;
use crate::util::cbor::{ cbor_map, cbor_map_iter, cbor_string, cbor_bytes };
use super::backoff::Backoff;
use super::request::{ RequestType, ResponseType, ResponseBuilderType };
use serde_cbor::Value as CborValue;
use super::failure::GeneralFailure;
use super::manager::RequestManager;
use crate::util::message::DataMessage;

pub struct DataResponse {
    data: HashMap<String,Vec<u8>>,
}

#[derive(Clone)]
pub struct DataCommandRequest {
    channel: Channel,
    name: String,
    region: Region
}

impl DataCommandRequest {
    pub fn new(channel: &Channel, name: &str, region: &Region) -> DataCommandRequest {
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

    pub async fn execute(self, manager: &RequestManager, priority: &PacketPriority, metrics: &MetricCollector) -> Result<Box<DataResponse>,DataMessage> {
        let mut backoff = Backoff::new(manager,&self.channel,priority);
        let mut out = backoff.backoff::<DataResponse,_>(self.clone()).await?
                .map_err(|e| DataMessage::DataUnavailable(self.channel.clone(),Box::new(e)));
        if let Ok(response) = &mut out {
            self.account(&response,metrics,priority);
        }
        out
    }
}

impl RequestType for DataCommandRequest {
    fn type_index(&self) -> u8 { 4 }
    fn serialize(&self, _channel: &Channel) -> Result<CborValue,DataMessage> {
        Ok(CborValue::Array(vec![self.channel.serialize()?,CborValue::Text(self.name.to_string()),self.region.serialize()?]))
    }
    fn to_failure(&self) -> Box<dyn ResponseType> {
        Box::new(GeneralFailure::new("data loading failed"))
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
