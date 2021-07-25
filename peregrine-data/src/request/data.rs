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

    pub async fn execute(self, mut manager: RequestManager) -> Result<Box<DataResponse>,DataMessage> {
        let mut backoff = Backoff::new();
        match backoff.backoff::<DataResponse,_,_>(
                                    &mut manager,self.clone(),&self.channel,PacketPriority::RealTime,|_| None).await? {
            Ok(d) => {
                Ok(d)
            },
            Err(e) => {
                // XXX and send via messagesender
                Err(DataMessage::DataUnavailable(self.channel.clone(),Box::new(e)))
            }
        }
    }
}

impl RequestType for DataCommandRequest {
    fn type_index(&self) -> u8 { 4 }
    fn serialize(&self) -> Result<CborValue,DataMessage> {
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
