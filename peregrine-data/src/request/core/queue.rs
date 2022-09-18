use peregrine_toolkit_async::sync::blocker::{Blocker};
use peregrine_toolkit::{log_extra};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use crate::core::channel::{Channel, ChannelIntegration, PacketPriority};
use crate::core::version::VersionMetadata;
use crate::{RequestPacket, ResponsePacket};
use crate::api::MessageSender;
use crate::run::{ PgCommander, add_task };
use crate::run::pgcommander::PgCommanderTaskSpec;
use crate::util::message::DataMessage;
use super::attemptmatch::AttemptMatch;
use super::packet::RequestPacketFactory;
use super::pendingattemptqueue::PendingAttemptQueue;
use super::sidecars::RequestSidecars;
use super::trafficcontrol::TrafficControl;

#[derive(Clone)]
pub struct RequestQueue {
    integration: Rc<Box<dyn ChannelIntegration>>,
    sidecars: RequestSidecars,
    channel: Channel,
    messages: MessageSender,
    pending_send: PendingAttemptQueue,
    priority: PacketPriority,
    packet_factory: RequestPacketFactory,
    traffic_control: TrafficControl
}

impl RequestQueue {
    pub(crate) fn new(commander: &PgCommander, realtime_lock: &Blocker, matcher: &AttemptMatch, sidecars: &RequestSidecars, integration: &Rc<Box<dyn ChannelIntegration>>, version: &VersionMetadata, channel: &Channel, priority: &PacketPriority, messages: &MessageSender, pacing: &[f64], cdr_priority: u8) -> Result<RequestQueue,DataMessage> {
        let batch_size = match priority {
            PacketPriority::RealTime => None, /* limitless */
            PacketPriority::Batch => Some(20) /* no more than 20 at a time */
        };
        let out = RequestQueue {
            integration: integration.clone(),
            sidecars: sidecars.clone(),
            channel: channel.clone(),
            messages: messages.clone(),
            pending_send: PendingAttemptQueue::new(batch_size),
            priority: priority.clone(),
            packet_factory: RequestPacketFactory::new(channel,version),
            traffic_control: TrafficControl::new(realtime_lock,priority,pacing)
        };
        out.start(commander,matcher,cdr_priority)?;
        Ok(out)
    }

    fn make_packet_sender(&self, packet: &RequestPacket, priority: &PacketPriority) -> Result<Pin<Box<dyn Future<Output=Result<ResponsePacket,DataMessage>>>>,DataMessage> {
        let channel = self.channel.clone();
        let integration = self.integration.clone();
        Ok(integration.get_sender(channel,priority.clone(),packet.clone()))
    }

    pub(crate) fn input_queue(&self) -> &PendingAttemptQueue { &self.pending_send }

    fn start(&self, commander: &PgCommander, matcher: &AttemptMatch, prio: u8) -> Result<(),DataMessage> {
        let name = format!("backend: '{}' {}",self.channel.to_string(),self.priority.to_string());
        let self2 = self.clone();
        let matcher = matcher.clone();
        add_task(&commander,PgCommanderTaskSpec {
            name,
            prio,
            timeout: None,
            slot: None,
            task: Box::pin(self2.main_loop(matcher)),
            stats: false
        });
        Ok(())
    }

    async fn build_packet(&self) -> Option<RequestPacket> {
        let mut packet = self.packet_factory.create();
        if !self.pending_send.add_to_packet(&mut packet).await {
            return None; /* queue closed */
        }
        Some(RequestPacket::new(packet))
    }

    async fn send_packet(&self, packet: &RequestPacket) -> Result<ResponsePacket,DataMessage> {
        let sender = self.make_packet_sender(packet,&self.priority)?;
        let lockout = self.traffic_control.await_permission().await;
        let response = sender.await?;
        drop(lockout);
        Ok(response)
    }

    async fn send_or_fail_packet(&self, packet: &RequestPacket) -> ResponsePacket {
        let res = self.send_packet(packet).await;
        self.traffic_control.notify_outcome(res.is_ok());
        if let Some(e) = &res.as_ref().err() {
            self.messages.send(DataMessage::PacketError(self.channel.clone(),e.to_string()));
        }
        res.ok().unwrap_or_else(|| packet.fail())
    }

    async fn process_responses(&self, matcher: &AttemptMatch, mut response: ResponsePacket) {
        let channel = self.channel.clone();
        let messages = self.messages.clone();
        self.sidecars.run(&response,&channel,&messages).await;
        for r in response.take_responses().drain(..) {
            if let Some(stream) = matcher.retrieve_attempt_by_response(&r) {
                let response = r.into_variety();
                stream.add(response);
            }
        }
    }

    async fn process_request(&self, matcher: &AttemptMatch, request: &mut RequestPacket) {
        let response = self.send_or_fail_packet(request).await;
        self.process_responses(matcher,response).await;
    }

    async fn main_loop(self, matcher: AttemptMatch) -> Result<(),DataMessage> {
        loop {
            let request = self.build_packet().await;
            if let Some(mut request) = request {
                self.process_request(&matcher,&mut request).await;
            } else {
                break;
            }
        }
        log_extra!("connection manager shutting down");
        Ok(())
    }
}
