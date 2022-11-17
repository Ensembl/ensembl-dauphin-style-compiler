use std::{sync::Arc, pin::Pin, fmt, any::Any, collections::HashMap};
use futures::Future;
use peregrine_toolkit::error::Error;
use crate::{MaxiRequest, PacketPriority, MaxiResponse, ChannelSender, DataAlgorithm};
use peregrine_toolkit::{identitynumber, hashable };

use super::channelintegration::{ChannelMessageDecoder};

identitynumber!(IDS);
hashable!(WrappedChannelSender,id);
#[derive(Clone)]
pub(crate) struct WrappedChannelSender {
    sender: Arc<dyn ChannelSender>,
    id: u64
}

impl fmt::Debug for WrappedChannelSender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WrappedChannelSender").field("id", &self.id).finish()
    }
}

impl WrappedChannelSender {
    pub(crate) fn new(sender: Arc<dyn ChannelSender>) -> WrappedChannelSender {
        WrappedChannelSender {
            sender: sender.clone(),
            id: IDS.next()
        }
    }

    pub(crate) fn id(&self) -> u64 { self.id }
}

impl ChannelSender for WrappedChannelSender {
    fn get_sender(&self, prio: &PacketPriority, data: MaxiRequest, decoder: ChannelMessageDecoder) -> Pin<Box<dyn Future<Output=Result<MaxiResponse,Error>>>> {
        self.sender.get_sender(prio,data,decoder)
    }

    fn deserialize_data(&self, payload: &dyn Any, bytes: Vec<u8>) -> Result<Option<HashMap<String,DataAlgorithm>>,String> {
        self.sender.deserialize_data(payload,bytes)
    }

    fn deserialize_index(&self, payload: &dyn Any, index: usize) -> Result<Option<Vec<u8>>,String> { 
        self.sender.deserialize_index(payload,index)
    }

    fn backoff(&self) -> bool {
        self.sender.backoff()
    }
}
