use crate::metric::datastreammetric::DatastreamMetricKey;
use crate::metric::datastreammetric::DatastreamMetricValue;
use crate::metric::metricreporter::MetricCollector;
use anyhow::{ anyhow as err };
use serde::Deserializer;
use serde::Serializer;
use serde::de::Visitor;
use serde::ser::SerializeSeq;
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use super::channel::{ Channel, PacketPriority };
use super::request::RequestType;
use crate::Region;
use super::backoff::Backoff;
use super::manager::RequestManager;
use crate::util::message::DataMessage;

pub struct ReceivedData(Arc<Vec<u8>>);

impl ReceivedData {
    pub fn len(&self) -> usize { self.0.len() }
    pub fn data(&self) -> &[u8] { &self.0 }
}

struct DataVisitor;

impl<'de> Visitor<'de> for DataVisitor {
    type Value = ReceivedData;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f,"some data") }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<ReceivedData,E> where E: serde::de::Error {
        Ok(ReceivedData(Arc::new(v.to_vec())))
    }
}

impl<'de> serde::Deserialize<'de> for ReceivedData {
    fn deserialize<D>(deserializer: D) -> Result<ReceivedData, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_bytes(DataVisitor)
    }
}
struct DatasetResponse(HashMap<String,ReceivedData>);

#[derive(Deserialize)]
pub struct DataResponse {
    data: DatasetResponse
}

struct DatasetResponseVisitor;

impl<'de> Visitor<'de> for DatasetResponseVisitor {
    type Value = DatasetResponse;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f,"a data response") }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error> where A: serde::de::MapAccess<'de> {
        let mut data = HashMap::new();
        while let Some((k,v)) = map.next_entry::<String,ReceivedData>()? {
            data.insert(k,v);
        }
        Ok(DatasetResponse(data))
    }
}

impl<'de> serde::Deserialize<'de> for DatasetResponse {
    fn deserialize<D>(deserializer: D) -> Result<DatasetResponse, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_map(DatasetResponseVisitor)
    }
}

pub(super) async fn do_data_request(channel: &Channel, name: &str, region: &Region, manager: &RequestManager, priority: &PacketPriority, metrics: &MetricCollector) -> Result<DataResponse,DataMessage> {
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
        for (name,data) in &response.data.0 {
            let key = DatastreamMetricKey::new(&self.name,name,self.region.scale().get_index(),priority.clone());
            let mut value = DatastreamMetricValue::empty();
            value.num_events += 1;
            value.total_size += data.len();
            metrics.add_datastream(&key,&value);
        }
    }

    async fn execute(self, manager: &RequestManager, priority: &PacketPriority, metrics: &MetricCollector) -> Result<DataResponse,DataMessage> {
        let mut backoff = Backoff::new(manager,&self.channel,priority);
        let r = backoff.backoff(RequestType::new_data(self.clone()), |v| {
            v.into_data()
        }).await?;
        self.account(&r,metrics,priority);
        Ok(r)
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

impl DataResponse {
    pub fn get(&self, name: &str) -> anyhow::Result<&ReceivedData> {
        self.data.0.get(name).ok_or_else(|| err!("no such data {}",name))
    }
}
