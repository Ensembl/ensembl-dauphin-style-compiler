use serde::{Serializer, ser::SerializeSeq};
use crate::{Region, core::channel::Channel, request::core::request::NewRequestVariant};

#[derive(Clone)]
pub(crate) struct DataCommandRequest {
    channel: Channel,
    name: String,
    region: Region
}

impl DataCommandRequest {
    pub(crate) fn new(channel: &Channel, name: &str, region: &Region) -> NewRequestVariant {
        NewRequestVariant::Data(DataCommandRequest {
            channel: channel.clone(),
            name: name.to_string(),
            region: region.clone()
        })
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
