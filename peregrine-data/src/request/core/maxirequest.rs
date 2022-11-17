use futures::Future;
use peregrine_toolkit::error::Error;
use serde::{Serialize};
use serde::ser::SerializeMap;
use std::pin::Pin;
use std::sync::Arc;
use crate::core::channel::channelintegration::{ChannelMessageDecoder};
use crate::core::channel::wrappedchannelsender::WrappedChannelSender;
use crate::{DataMessage, ChannelSender, BackendNamespace};
use crate::core::version::VersionMetadata;
use super::maxiresponse::MaxiResponse;
use super::packet::{RequestPacketFactory, RequestPacketBuilder};
use super::minirequest::MiniRequestAttempt;

#[derive(Clone)]
pub struct MaxiRequest {
    factory: RequestPacketFactory,
    requests: Arc<Vec<MiniRequestAttempt>>,
}

impl MaxiRequest {
    pub fn new(builder: RequestPacketBuilder) -> MaxiRequest {
        MaxiRequest {
            factory: builder.factory.clone(),
            requests: Arc::new(builder.requests.clone()),
        }
    }

    pub fn fail(&self, extra: &str) -> MaxiResponse {
        let mut response = MaxiResponse::empty(&self.factory.channel);
        for r in self.requests.iter() {
            response.add_response(r.fail(extra));
        }
        response
    }

    pub fn requests(&self) -> &[MiniRequestAttempt] { &self.requests }
    pub fn channel(&self) -> &BackendNamespace { &self.factory.channel }
    pub fn metadata(&self) -> &VersionMetadata { &self.factory.metadata }

    pub(crate) fn sender(&self, sender: &WrappedChannelSender) -> Result<Pin<Box<dyn Future<Output=Result<MaxiResponse,Error>>>>,DataMessage> {
        let decoder = ChannelMessageDecoder::new(sender);
        Ok(sender.get_sender(&self.factory.priority,self.clone(),decoder))
    }
}

impl Serialize for MaxiRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        let mut map = serializer.serialize_map(Some(3))?;
        map.serialize_entry("channel",self.channel())?;
        map.serialize_entry("requests",self.requests())?;
        map.serialize_entry("version",self.metadata())?;
        map.end()
    }
}
