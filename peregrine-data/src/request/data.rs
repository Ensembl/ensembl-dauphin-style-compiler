use crate::index::metricreporter::DatastreamMetric;
use crate::index::metricreporter::MetricCollector;
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
    data: HashMap<String,Vec<u8>>
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

    fn account(&self, response: &DataResponse, metrics: &MetricCollector) {
        let mut datapoint = DatastreamMetric::empty(&self.name,self.region.scale().get_index());
        datapoint.num_events += 1;
        datapoint.total_size += response.data.iter().map(|(k,v)| k.len()+v.len()).sum::<usize>();
        metrics.add_datastream(&datapoint);
    }

    pub async fn execute(self, mut manager: RequestManager, priority: &PacketPriority, metrics: &MetricCollector) -> Result<Box<DataResponse>,DataMessage> {
        let mut backoff = Backoff::new();
        let out = backoff.backoff::<DataResponse,_,_>(
                                    &mut manager,self.clone(),&self.channel,priority.clone(),|_| None).await?
                .map_err(|e| DataMessage::DataUnavailable(self.channel.clone(),Box::new(e)));
        if let Ok(response) = &out {
            self.account(&response,metrics);
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
