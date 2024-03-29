use std::{sync::Arc, pin::Pin, any::Any, collections::HashMap};
use futures::{Future};
use peregrine_toolkit::error::Error;
use crate::{MaxiRequest, PacketPriority, MaxiResponse, BackendNamespace, MaxiResponseDeserialize, core::{dataalgorithm::DataAlgorithm}};
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

pub fn null_payload() -> Arc<dyn Any> { Arc::new(()) }

pub trait ChannelSender {
    fn get_sender(&self, prio: &PacketPriority, data: MaxiRequest, decoder: ChannelMessageDecoder) -> Pin<Box<dyn Future<Output=Result<MaxiResponse,Error>>>>;
    fn deserialize_data(&self, _payload: &dyn Any, _bytes: Vec<u8>) -> Result<Option<HashMap<String,DataAlgorithm>>,String> { Ok(None) }
    fn deserialize_index(&self, _payload: &dyn Any, _index: usize) -> Result<Option<Vec<u8>>,String> { Ok(None) }
    fn backoff(&self) -> bool;
}

pub trait ChannelIntegration {
    fn make_channel(&self, name: &str) -> Option<(Arc<dyn ChannelSender>,Option<BackendNamespace>)>;
}
