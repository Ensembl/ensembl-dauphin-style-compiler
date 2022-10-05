use std::{sync::Arc, pin::Pin};
use futures::Future;
use crate::{RequestPacket, PacketPriority, ResponsePacket, DataMessage, BackendNamespace};

pub trait ChannelSender {
    fn get_sender(&self, prio: &PacketPriority, data: RequestPacket) -> Pin<Box<dyn Future<Output=Result<ResponsePacket,DataMessage>>>>;
}

pub trait ChannelIntegration {
    fn make_channel(&self, name: &str) -> Option<(Arc<dyn ChannelSender>,Option<BackendNamespace>)>;
}
