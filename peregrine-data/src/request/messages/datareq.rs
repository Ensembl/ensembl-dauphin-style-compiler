use crate::{Region, core::channel::Channel, request::core::request::{RequestVariant}};
use serde_cbor::Value as CborValue;

#[derive(Clone)]
pub(crate) struct DataReq {
    channel: Channel,
    name: String,
    region: Region
}

impl DataReq {
    pub(crate) fn new(channel: &Channel, name: &str, region: &Region) -> RequestVariant {
        RequestVariant::Data(DataReq {
            channel: channel.clone(),
            name: name.to_string(),
            region: region.clone()
        })
    }

    pub(crate) fn encode(&self) -> CborValue {
        CborValue::Array(vec![
            self.channel.encode(),
            CborValue::Text(self.name.to_string()),
            self.region.encode()
        ])
    }
}
