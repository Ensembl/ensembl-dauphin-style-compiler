use commander::cdr_timer;
use peregrine_toolkit_async::sync::{pacer::Pacer, blocker::{Blocker, Lockout}};

use crate::PacketPriority;

#[derive(Clone)]
pub(crate) struct TrafficControl {
    pacer: Pacer<f64>,
    high_priority: bool,
    realtime_lock: Blocker
}

impl TrafficControl {
    pub(crate) fn new(realtime_lock: &Blocker, priority: &PacketPriority, pacing: &[f64]) -> TrafficControl {
        TrafficControl {
            pacer: Pacer::new(pacing),
            high_priority: if let PacketPriority::RealTime = priority { true } else { false },
            realtime_lock: realtime_lock.clone()
        }
    }

    pub(crate) fn notify_outcome(&self, success: bool) {
        self.pacer.report(success);
    }

    pub(crate) async fn await_permission(&self) -> Option<Lockout> {
        let wait = self.pacer.get();
        cdr_timer(wait).await;
        if self.high_priority {
            Some(self.realtime_lock.lock())
        } else {
            self.realtime_lock.wait().await;
            None
        }
    }
}
