use commander::cdr_timer;
use peregrine_toolkit_async::sync::blocker::{Blocker, Lockout};
use peregrine_toolkit::{lock, log_extra};
use commander::{ CommanderStream, cdr_add_timer };
use peregrine_toolkit_async::sync::pacer::Pacer;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::{ Arc, Mutex };
use crate::core::channel::{Channel, ChannelIntegration, PacketPriority};
use crate::core::version::VersionMetadata;
use crate::request::core::manager::PayloadReceiver;
use crate::request::core::packet::RequestPacketBuilder;
use crate::{RequestPacket, ResponsePacket};
use crate::api::MessageSender;
use crate::run::{ PgCommander, add_task };
use crate::run::pgcommander::PgCommanderTaskSpec;
use crate::util::message::DataMessage;

use super::manager::PayloadReceiverCollection;
use super::request::BackendRequestAttempt;
use super::response::BackendResponse;

struct RequestQueueData {
    version: VersionMetadata,
    receiver: PayloadReceiverCollection,
    pending_send: CommanderStream<Option<(BackendRequestAttempt,CommanderStream<BackendResponse>)>>,
    integration: Rc<Box<dyn ChannelIntegration>>,
    channel: Channel,
    priority: PacketPriority,
    pacer: Pacer<f64>,
    timeout: Option<f64>,
    messages: MessageSender,
    realtime_block: Option<Blocker>,
    realtime_block_check: Option<Blocker>
}

impl RequestQueueData {
    fn make_packet_sender(&self, packet: &RequestPacket) -> Result<Pin<Box<dyn Future<Output=Result<ResponsePacket,DataMessage>>>>,DataMessage> {
        let channel = self.channel.clone();
        let priority = self.priority.clone();
        let integration = self.integration.clone();
        Ok(integration.get_sender(channel,priority,packet.clone()))
    }

    fn report<T>(&self, msg: Result<T,DataMessage>) -> Result<T,DataMessage> {
        if let Some(ref e) = msg.as_ref().err() {
            self.messages.send(DataMessage::PacketError(self.channel.clone(),e.to_string()));
        }
        msg
    }

    fn acquire_realtime_lock(&self) -> Option<Lockout> {
        self.realtime_block.as_ref().map(|x| x.lock())
    }

    fn get_blocker(&self) -> Option<Blocker> {
        self.realtime_block_check.as_ref().cloned()
    }

    fn set_timeout(&mut self, timeout: f64) {
        self.timeout = Some(timeout);
    }

    fn timeout(&self, streams: Vec<(BackendResponse,CommanderStream<BackendResponse>)>) {
        if let Some(timeout) = self.timeout {
            for (response,stream) in streams {
                let stream = stream.clone();
                let channel = self.channel.clone();
                let messages = self.messages.clone();
                cdr_add_timer(timeout, move || {
                    if stream.add_first(response) {
                        messages.send(DataMessage::BackendTimeout(channel.clone()));
                    }
                });
            }
        }
    }
}

#[derive(Clone)]
pub struct RequestQueue(Arc<Mutex<RequestQueueData>>);

impl RequestQueue {
    pub fn new(commander: &PgCommander, receiver: &PayloadReceiverCollection, integration: &Rc<Box<dyn ChannelIntegration>>, version: &VersionMetadata, channel: &Channel, priority: &PacketPriority, messages: &MessageSender, pacing: &[f64], cdr_priority: u8) -> Result<RequestQueue,DataMessage> {
        let out = RequestQueue(Arc::new(Mutex::new(RequestQueueData {
            receiver: receiver.clone(),
            pending_send: CommanderStream::new(),
            integration: integration.clone(),
            channel: channel.clone(),
            priority: priority.clone(),
            timeout: None,
            messages: messages.clone(),
            pacer: Pacer::new(pacing),
            version: version.clone(),
            realtime_block: None,
            realtime_block_check: None
        })));
        out.start(commander,cdr_priority)?;
        Ok(out)
    }
    
    pub(super) fn set_realtime_block(&mut self, blocker: &Blocker) {
        self.0.lock().unwrap().realtime_block = Some(blocker.clone());
    }

    pub(super) fn set_realtime_check(&mut self, blocker: &Blocker) {
        self.0.lock().unwrap().realtime_block_check = Some(blocker.clone());
    }

    pub(crate) fn queue_command(&mut self, request: BackendRequestAttempt, stream: CommanderStream<BackendResponse>) {
        lock!(self.0).pending_send.add(Some((request,stream)));
    }

    fn start(&self, commander: &PgCommander, prio: u8) -> Result<(),DataMessage> {
        let data = lock!(self.0);
        let name = format!("backend: '{}' {}",data.channel.to_string(),data.priority.to_string());
        drop(data);
        let self2 = self.clone();
        add_task(&commander,PgCommanderTaskSpec {
            name,
            prio,
            timeout: None,
            slot: None,
            task: Box::pin(self2.main_loop()),
            stats: false
        });
        Ok(())
    }

    async fn build_packet(&self) -> (Option<RequestPacket>,HashMap<u64,CommanderStream<BackendResponse>>) {
        let data = lock!(self.0);
        let pending = data.pending_send.clone();
        let priority = data.priority.clone();
        let channel = data.channel.clone();
        let version = data.version.clone();
        drop(data);
        let mut requests = match priority {
            PacketPriority::RealTime => { pending.get_multi(None).await },
            PacketPriority::Batch => { pending.get_multi(Some(20)).await }
        };
        let mut packet = RequestPacketBuilder::new(&channel);
        let mut streams = HashMap::new();
        let mut timeouts = vec![];
        for data in requests.drain(..) {
            if let Some((r,c)) = data {
                streams.insert(r.message_id(),c.clone());
                timeouts.push((r.to_failure(),c));
                packet.add(r);
            } else {
                return (None,streams);
            }
        }
        lock!(self.0).timeout(timeouts);
        (Some(RequestPacket::new(packet,&version)),streams)
    }

    fn get_blocker(&self) -> Option<Blocker> {
        lock!(self.0).get_blocker()
    }

    async fn pace(&self) {
        let state = lock!(self.0);
        let wait = state.pacer.get();
        drop(state);
        cdr_timer(wait).await;
    }

    async fn send_packet(&self, packet: &RequestPacket) -> Result<ResponsePacket,DataMessage> {
        let sender = lock!(self.0).make_packet_sender(packet)?;
        self.pace().await;
        if let Some(blocker) = self.get_blocker() {
            blocker.wait().await;
        }
        let lockout = lock!(self.0).acquire_realtime_lock();
        let response = sender.await?;
        drop(lockout);
        Ok(response)
    }

    async fn send_or_fail_packet(&self, packet: &RequestPacket) -> ResponsePacket {
        let res = self.send_packet(packet).await;
        let state = lock!(self.0);
        match state.report(res) {
            Ok(r) => {
                state.pacer.report(true);
                r
            },
            Err(_) => {
                state.pacer.report(false);
                packet.fail()
            }
        }
    }

    async fn process_responses(&self, response: ResponsePacket, streams: &mut HashMap<u64,CommanderStream<BackendResponse>>) {
        let channel = lock!(self.0).channel.clone();
        let itn = lock!(self.0).integration.clone();
        let receiver = lock!(self.0).receiver.clone();
        let messages = lock!(self.0).messages.clone();
        let mut response = receiver.receive(&channel,response,&itn,&messages).await;
        for r in response.take_responses().drain(..) {
            let id = r.message_id();
            if let Some(stream) = streams.remove(&id) {
                let response = r.into_variety();
                stream.add(response);
            }
        }
    }

    async fn process_request(&self, request: &mut RequestPacket, streams: &mut HashMap<u64,CommanderStream<BackendResponse>>) {
        let response = self.send_or_fail_packet(request).await;
        self.process_responses(response,streams).await;
    }

    pub(crate) fn set_timeout(&mut self, timeout: f64) {
        lock!(self.0).set_timeout(timeout);
    }

    pub(crate) fn shutdown(&mut self) {
        lock!(self.0).pending_send.add(None);
    }

    async fn main_loop(self) -> Result<(),DataMessage> {
        loop {
            let (request,mut streams) = self.build_packet().await;
            if let Some(mut request) = request {
                self.process_request(&mut request,&mut streams).await;
            } else {
                break;
            }
        }
        log_extra!("connection manager shutting down");
        Ok(())
    }
}
