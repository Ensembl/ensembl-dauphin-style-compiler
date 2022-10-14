use peregrine_toolkit::error::Error;
use peregrine_toolkit_async::sync::blocker::{Blocker};
use peregrine_toolkit::{log_extra};
use crate::core::channel::wrappedchannelsender::WrappedChannelSender;
use crate::core::version::VersionMetadata;
use crate::{PacketPriority, BackendNamespace, MaxiResponse, MaxiRequest};
use crate::api::MessageSender;
use crate::run::{ PgCommander, add_task };
use crate::run::pgcommander::PgCommanderTaskSpec;
use crate::util::message::DataMessage;
use super::attemptmatch::{AttemptMatch};
use super::packet::{RequestPacketFactory};
use super::pendingattemptqueue::PendingAttemptQueue;
use super::sidecars::RequestSidecars;
use super::trafficcontrol::TrafficControl;

#[derive(Clone,PartialEq,Eq,Hash)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub(crate) struct QueueKey {
    pub(super) sender: WrappedChannelSender,
    pub(super) priority: PacketPriority,
    pub(super) name: Option<BackendNamespace>
}

impl QueueKey {
    pub(super) fn new(sender: &WrappedChannelSender, priority: &PacketPriority, name: &Option<BackendNamespace>) -> QueueKey {
        QueueKey {
            sender: sender.clone(),
            priority: priority.clone(),
            name: name.clone()
        }
    }
}

#[derive(Clone)]
pub struct RequestQueue {
    key: QueueKey,
    name: String,
    messages: MessageSender,
    pending_send: PendingAttemptQueue,
    packet_factory: RequestPacketFactory,
    traffic_control: TrafficControl
}

impl RequestQueue {
    pub(crate) fn new(key: &QueueKey, commander: &PgCommander, realtime_lock: &Blocker, matcher: &AttemptMatch, sidecars: &RequestSidecars, version: &VersionMetadata, messages: &MessageSender) -> Result<RequestQueue,Error> {
        let batch_size = match key.priority {
            PacketPriority::RealTime => None, /* limitless */
            PacketPriority::Batch => Some(20) /* no more than 20 at a time */
        };
        let out = RequestQueue {
            key: key.clone(),
            messages: messages.clone(),
            pending_send: PendingAttemptQueue::new(batch_size),
            name: format!("backend: '{:?}' {}",key.sender.id(),key.priority.to_string()),
            packet_factory: RequestPacketFactory::new(&BackendNamespace::or_missing(&key.name),&key.priority,version),
            traffic_control: TrafficControl::new(realtime_lock,&key.priority,&key.priority.get_pace())
        };
        out.start(commander,matcher,sidecars,key.priority.cdr_priority())?;
        Ok(out)
    }

    pub(crate) fn input_queue(&self) -> &PendingAttemptQueue { &self.pending_send }

    fn start(&self, commander: &PgCommander, matcher: &AttemptMatch, sidecars: &RequestSidecars, prio: u8) -> Result<(),Error> {
        let self2 = self.clone();
        let matcher = matcher.clone();
        let sidecars = sidecars.clone();
        add_task(&commander,PgCommanderTaskSpec {
            name: self.name.clone(),
            prio,
            timeout: None,
            slot: None,
            task: Box::pin(self2.main_loop(matcher,sidecars)),
            stats: false
        });
        Ok(())
    }

    async fn build_packet(&self) -> Option<MaxiRequest> {
        let mut packet = self.packet_factory.create();
        if !self.pending_send.add_to_packet(&mut packet).await {
            return None; /* queue closed */
        }
        Some(MaxiRequest::new(packet))
    }

    async fn send_packet(&self, packet: &MaxiRequest) -> Result<MaxiResponse,DataMessage> {
        let sender = packet.sender(&self.key.sender)?;
        let lockout = self.traffic_control.await_permission().await;
        let response = sender.await.map_err(|x| DataMessage::XXXTransitional(x))?;
        drop(lockout);
        Ok(response)
    }

    async fn send_or_fail_packet(&self, packet: &MaxiRequest) -> MaxiResponse {
        let res = self.send_packet(packet).await;
        self.traffic_control.notify_outcome(res.is_ok());
        if let Some(e) = &res.as_ref().err() {
            self.messages.send(DataMessage::PacketError(self.name.clone(),e.to_string()));
        }
        res.ok().unwrap_or_else(|| packet.fail("network/backend failed"))
    }

    async fn process_request(&self, matcher: &AttemptMatch, sidecars: &RequestSidecars, request: &mut MaxiRequest) {
        let mut response = self.send_or_fail_packet(request).await;
        sidecars.run(&response,&response.channel(),&self.messages).await;
        for r in response.take_responses().drain(..) {
            if let Some(stream) = matcher.retrieve_callback_by_response(&r) {
                stream.add(r);
            }
        }
    }

    async fn main_loop(self, matcher: AttemptMatch, sidecars: RequestSidecars) -> Result<(),DataMessage> {
        loop {
            let request = self.build_packet().await;
            if let Some(mut request) = request {
                self.process_request(&matcher,&sidecars,&mut request).await;
            } else {
                break;
            }
        }
        log_extra!("connection manager shutting down");
        Ok(())
    }
}
