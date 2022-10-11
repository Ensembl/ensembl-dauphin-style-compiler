use std::{sync::Arc, pin::Pin, any::Any};
use futures::{Future};
use peregrine_toolkit::error::Error;
use crate::{MaxiRequest, PacketPriority, MaxiResponse, BackendNamespace, MaxiResponseDeserialize};
use serde_cbor::Value as CborValue;

use super::wrappedchannelsender::WrappedChannelSender;

pub trait ChannelResponse {
    fn take_response(&mut self) -> MaxiResponse; // guarantee: will only be called once
}

pub struct TrivialChannelResponse(Option<MaxiResponse>);
impl TrivialChannelResponse {
    pub fn new(maxi: MaxiResponse) -> TrivialChannelResponse {
        TrivialChannelResponse(Some(maxi))
    }
}

impl ChannelResponse for TrivialChannelResponse {
    fn take_response(&mut self) -> MaxiResponse { self.0.take().unwrap() }
}

pub struct ChannelMessageDecoder {
    sender: WrappedChannelSender
}

impl ChannelMessageDecoder {
    pub(crate) fn new(sender: &WrappedChannelSender) -> ChannelMessageDecoder {
        ChannelMessageDecoder { sender: sender.clone() }
    }

    pub fn serde_deserialize_maxi(&self, payload: Arc<dyn Any>) -> MaxiResponseDeserialize {
        MaxiResponseDeserialize(self.sender.clone(),payload)
    }
}

pub trait ChannelSender {
    fn get_sender(&self, prio: &PacketPriority, data: MaxiRequest, decoder: ChannelMessageDecoder) -> Pin<Box<dyn Future<Output=Result<MaxiResponse,Error>>>>;
    fn deserialize_data(&self, _payload: &dyn Any, _bytes: Vec<u8>) -> Result<Option<Vec<(String,CborValue)>>,String> { Ok(None) }
    fn deserialize_index(&self, _payload: &dyn Any, _index: usize) -> Result<Option<CborValue>,String> { Ok(None) }
}

pub trait ChannelIntegration {
    fn make_channel(&self, name: &str) -> Option<(Arc<dyn ChannelSender>,Option<BackendNamespace>)>;
}
