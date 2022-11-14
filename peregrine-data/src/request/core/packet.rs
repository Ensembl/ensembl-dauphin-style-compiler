use crate::{PacketPriority, BackendNamespace};
use crate::core::version::VersionMetadata;
use super::minirequest::MiniRequestAttempt;

#[cfg(debug_big_requests)]
use peregrine_toolkit::{warn};

#[derive(Clone)]
pub struct RequestPacketFactory {
    pub(super) channel: BackendNamespace,
    pub(super) priority: PacketPriority,
    pub(super) metadata: VersionMetadata,
}

impl RequestPacketFactory {
    pub fn new(channel: &BackendNamespace, priority: &PacketPriority, metadata: &VersionMetadata) -> RequestPacketFactory {
        RequestPacketFactory {
            channel: channel.clone(),
            priority: priority.clone(),
            metadata: metadata.clone()
        }
    }

    pub fn create(&self) -> RequestPacketBuilder {
        RequestPacketBuilder::new(&self)
    }
}

pub struct RequestPacketBuilder {
    pub(super) factory: RequestPacketFactory,
    pub(super) requests: Vec<MiniRequestAttempt>
}

impl RequestPacketBuilder {
    fn new(factory: &RequestPacketFactory) -> RequestPacketBuilder {
        RequestPacketBuilder {
            factory: factory.clone(),
            requests: vec![]
        }
    }

    pub fn add(&mut self, request: MiniRequestAttempt) {
        self.requests.push(request);
    }
}
