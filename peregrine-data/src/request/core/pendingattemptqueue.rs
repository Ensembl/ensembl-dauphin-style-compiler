use commander::CommanderStream;
use super::{minirequest::MiniRequestAttempt, packet::RequestPacketBuilder};

#[derive(Clone)]
pub(crate) struct PendingAttemptQueue {
    batch_size: Option<usize>,
    pending: CommanderStream<Option<MiniRequestAttempt>>
}

impl PendingAttemptQueue {
    pub(crate) fn new(batch_size: Option<usize>) -> PendingAttemptQueue {
        PendingAttemptQueue {
            pending: CommanderStream::new(),
            batch_size
        }
    }

    pub(crate) fn add(&self, attempt: MiniRequestAttempt) {
        self.pending.add(Some(attempt));
    }

    pub(crate) fn close(&self) {
        self.pending.add(None);
    }

    pub(crate) async fn add_to_packet(&self, packet: &mut RequestPacketBuilder) -> bool {
        let requests = self.pending.get_multi(self.batch_size).await;
        for item in requests {
            if let Some(item) = item {
                packet.add(item);
            } else {
                return false; // close was received
            }
        }
        true
    }
}
