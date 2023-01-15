use anyhow::anyhow as err;
use peregrine_toolkit::serdetools::{st_field, ByteData };
use serde::de::{Visitor, MapAccess, DeserializeSeed, self, IgnoredAny, };
use serde::{Deserializer};
use std::any::Any;
use std::fmt;
use std::sync::Arc;
use std::{collections::HashMap};
use crate::core::dataalgorithm::DataAlgorithm;
use crate::{ChannelSender, PacketPriority};
use crate::core::channel::wrappedchannelsender::WrappedChannelSender;
use crate::request::core::miniresponse::MiniResponseVariety;
use crate::{metric::datastreammetric::PacketDatastreamMetricBuilder};
use crate::core::data::ReceivedData;

pub struct DataRes {
    data: HashMap<String,ReceivedData>,
    invariant: bool
}

impl DataRes {
    pub fn new(data: HashMap<String,ReceivedData>, invariant: bool) -> DataRes {
        DataRes { data, invariant }
    }

    pub(crate) fn account(&self, account_builder: &PacketDatastreamMetricBuilder) {
        for (name,data) in &self.data {
            account_builder.add(name,data.len());
        }
    }

    fn get2(&self, name: &str) -> anyhow::Result<&ReceivedData> {
        self.data.get(name)
            .ok_or_else(|| err!("no such data {}: have {}",
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
        let mut data : Option<HashMap<String,DataAlgorithm>> = None;
        let mut invariant = false;
        while let Some(key) = access.next_key()? {
            /* undo domain-specific compression */
            match key {
                "data" => { 
                    let bytes : ByteData = access.next_value()?;
                    data = self.0.deserialize_data(&self.1,bytes.data).map_err(|e| de::Error::custom(e))?;
                },
                "values" => {
                    data = Some(access.next_value()?);
                },
                "__invariant" => { invariant = access.next_value()? },
                _ => { let _ : IgnoredAny = access.next_value()?; }
            }
        }
        let mut data = st_field("data",data)?;
        let data : HashMap<String,ReceivedData> = data.drain().map(|(k,v)| {
            Ok((k.clone(),v.to_received_data().map_err(|_e| {
                de::Error::custom(&format!("wrong type for field: {} {:?}",k,v))
            })?))
        }).collect::<Result<_,_>>()?;
        //debug_log!("data: {:?}",data);
        Ok(DataRes {
            data,
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
pub struct DataResponse(Arc<DataRes>,PacketPriority);

impl DataResponse {
    pub fn new(res: DataRes, priority: PacketPriority) -> DataResponse {
        DataResponse(Arc::new(res),priority)
    }

    pub(crate) fn account(&self, account_builder: &PacketDatastreamMetricBuilder) {
        self.0.account(account_builder);
    }

    pub fn get2(&self, name: &str) -> anyhow::Result<&ReceivedData> { self.0.get2(name) }

    pub(crate) fn is_invariant(&self) -> bool { self.0.is_invariant() }
    pub(crate) fn original_priority(&self) -> PacketPriority { self.1.clone() }
}
