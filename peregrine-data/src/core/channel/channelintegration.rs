use std::{sync::Arc, pin::Pin};
use futures::Future;
use peregrine_toolkit::error::Error;
use crate::{MaxiRequest, PacketPriority, ResponsePacket, BackendNamespace};

pub trait ChannelSender {
    fn get_sender(&self, prio: &PacketPriority, data: MaxiRequest) -> Pin<Box<dyn Future<Output=Result<ResponsePacket,Error>>>>;
}

pub trait ChannelIntegration {
    fn make_channel(&self, name: &str) -> Option<(Arc<dyn ChannelSender>,Option<BackendNamespace>)>;
}
