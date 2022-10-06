use std::{sync::Arc, pin::Pin, fmt};
use futures::Future;
use peregrine_toolkit::error::Error;
use crate::{RequestPacket, PacketPriority, ResponsePacket, DataMessage, ChannelSender};
use lazy_static::lazy_static;
use identitynumber::{identitynumber, hashable };

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
    fn get_sender(&self, prio: &PacketPriority, data: RequestPacket) -> Pin<Box<dyn Future<Output=Result<ResponsePacket,Error>>>> {
        self.sender.get_sender(prio,data)
    }
}
