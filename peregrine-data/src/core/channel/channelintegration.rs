use std::{sync::Arc, pin::Pin};
use futures::Future;
use crate::{Channel, RequestPacket, PacketPriority, ResponsePacket, DataMessage};

pub trait ChannelSender {
    fn get_sender(&self, prio: &PacketPriority, data: RequestPacket) -> Pin<Box<dyn Future<Output=Result<ResponsePacket,DataMessage>>>>;
}

pub trait ChannelIntegration {
    fn set_timeout(&self, channel: &Channel, timeout: f64);
    fn make_sender(&self, channel: &Channel) -> Option<Arc<dyn ChannelSender>>;
}
