use anyhow::anyhow as err;
use peregrine_toolkit::serdetools::{st_field, st_err, ByteData };
use serde::de::{Visitor, MapAccess, DeserializeSeed, self, };
use serde::{Deserializer};
use std::any::Any;
use std::fmt;
use std::sync::Arc;
use std::{collections::HashMap};
use crate::core::dataalgorithm::DataAlgorithm;
use crate::{ChannelSender};
use crate::core::channel::wrappedchannelsender::WrappedChannelSender;
use crate::request::core::response::MiniResponseVariety;
use crate::{metric::datastreammetric::PacketDatastreamMetricBuilder};
use crate::core::data::ReceivedData;

pub struct DataRes {
    data: HashMap<String,ReceivedData>,
    data2: HashMap<String,ReceivedData>,
    invariant: bool
}

impl DataRes {
    pub(crate) fn account(&self, account_builder: &PacketDatastreamMetricBuilder) {
        for (name,data) in &self.data {
            account_builder.add(name,data.len());
        }
    }

    fn get(&self, name: &str) -> anyhow::Result<&ReceivedData> {
        self.data.get(name).ok_or_else(|| err!("no such data {}: have {}",
            name,
            self.data.keys().cloned().collect::<Vec<_>>().join(", ")
        ))
    }

    fn is_invariant(&self) -> bool { self.invariant }
}

struct DataVisitor(WrappedChannelSender,Arc<dyn Any>);

impl<'de> Visitor<'de> for DataVisitor {
    type Value = DataRes;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a DataRes")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where M: MapAccess<'de> {
        let mut data : Option<HashMap<String,Vec<u8>>> = None;
        let mut data2 : Option<HashMap<String,DataAlgorithm>> = None;
        let mut indexes : Option<HashMap<String,usize>> = None;
        let mut indexes2 : Option<HashMap<String,usize>> = None;
        let mut invariant = false;
        while let Some(key) = access.next_key()? {
            /* undo domain-specific compression */
            match key {
                /* values are present as some domain-compressed thing */
                "data" => { 
                    let bytes : ByteData = access.next_value()?;
                    let values = self.0.deserialize_data(&self.1,bytes.data).map_err(|e| de::Error::custom(e))?;
                    data = Some(st_field("data deserializer",values)?.drain(..).collect());
                },
                "values" => {
                    /* values are present as map From strings to bytes */
                    data = Some(access.next_value()?);
                },
                "indexes" => {
                    /* values are present as indexes into an index stream */
                    indexes = Some(access.next_value()?);
                },
                /***/
                "data2" => { 
                    let bytes : ByteData = access.next_value()?;
                    let values = self.0.deserialize_data2(&self.1,bytes.data).map_err(|e| de::Error::custom(e))?;
                    data2 = Some(st_field("data deserializer",values)?.drain(..).collect());
                },
                "values2" => {
                    /* values are present as map From strings to bytes */
                    data2 = Some(access.next_value()?);
                },
                "indexes2" => {
                    /* values are present as indexes into an index stream */
                    indexes2 = Some(access.next_value()?);
                },
                /***/
                "__invariant" => { invariant = access.next_value()? },
                _ => {}
            }
        }
        if let Some(mut indexes) = indexes {
            let values = indexes.drain().map(|(k,v)| {
                let value = self.0.deserialize_index(self.1.as_ref(),v).map_err(|e| de::Error::custom(e))?;
                Ok((k,st_field("index deserial",value)?))
            }).collect::<Result<HashMap<_,_>,_>>()?;
            data = Some(values)
        }
        let mut data = st_field("data",data)?;
        let mut data2 = data2.unwrap_or_else(|| HashMap::new());
        let data = st_err(data.drain().map(|(k,v)| {
            Ok((k,ReceivedData::new_bytes(v)))
        }).collect::<Result<HashMap<String,ReceivedData>,String>>(),"corrupt payload/C")?;
        let data2 = data2.drain().map(|(k,v)| {
            Ok((k,v.to_received_data()?))
        }).collect::<Result<_,()>>().map_err(|_| de::Error::custom("cannot create data"))?;
        Ok(DataRes {
            data, 
            data2,
            invariant
        })
    }
}

pub(crate) struct DataResDeserialize(pub WrappedChannelSender,pub Arc<dyn Any>);

impl<'de> DeserializeSeed<'de> for DataResDeserialize {
    type Value = DataRes;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where D: Deserializer<'de> {
        deserializer.deserialize_map(DataVisitor(self.0.clone(),self.1.clone()))
    }
}

impl MiniResponseVariety for DataRes {
    fn description(&self) -> &str { "data" }
    fn total_size(&self) -> usize { self.data.values().map(|x| x.len()).sum() }

    fn component_size(&self) -> Vec<(String,usize)> {
        self.data.iter().map(|(k,v)| (k.to_string(),v.len())).collect::<Vec<_>>()
    }
}

#[derive(Clone)]
pub struct DataResponse(Arc<DataRes>);

impl DataResponse {
    pub fn new(res: DataRes) -> DataResponse {
        DataResponse(Arc::new(res))
    }

    pub(crate) fn account(&self, account_builder: &PacketDatastreamMetricBuilder) {
        self.0.account(account_builder);
    }

    pub fn get(&self, name: &str) -> anyhow::Result<&ReceivedData> { self.0.get(name) }

    pub(crate) fn is_invariant(&self) -> bool { self.0.is_invariant() }
}
