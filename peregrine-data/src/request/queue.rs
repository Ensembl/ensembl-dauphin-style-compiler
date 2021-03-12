use anyhow::{ Context };
use crate::lock;
use blackbox::{ blackbox_count, blackbox_log };
use commander::{ CommanderStream, cdr_add_timer };
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::{ Arc, Mutex };
use crate::api::MessageSender;
use super::bootstrap::BootstrapResponseBuilderType;
use super::data::DataResponseBuilderType;
use super::failure::GeneralFailureBuilderType;
use super::program::{ ProgramResponseBuilderType };
use super::channel::{ Channel, PacketPriority, ChannelIntegration };
use super::manager::{ PayloadReceiver, PayloadReceiverCollection };
use super::packet::{ RequestPacket, ResponsePacket, ResponsePacketBuilder, ResponsePacketBuilderBuilder };
use super::request::{ CommandRequest, ResponseType };
use crate::run::{ PgCommander };
use crate::run::pgcommander::PgCommanderTaskSpec;
use super::stick::StickResponseBuilderType;
use super::stickauthority::StickAuthorityResponseBuilderType;
use serde_cbor::Value as CborValue;
use crate::util::message::DataMessage;

fn register_responses() -> ResponsePacketBuilder {
    let mut rspbb = ResponsePacketBuilderBuilder::new();
    rspbb.register(0,Box::new(BootstrapResponseBuilderType()));
    rspbb.register(1,Box::new(GeneralFailureBuilderType()));
    rspbb.register(2,Box::new(ProgramResponseBuilderType()));
    rspbb.register(3,Box::new(StickResponseBuilderType()));
    rspbb.register(4,Box::new(StickAuthorityResponseBuilderType()));
    rspbb.register(5,Box::new(DataResponseBuilderType()));
    rspbb.build()
}

struct RequestQueueData {
    receiver: PayloadReceiverCollection,
    builder: ResponsePacketBuilder,
    pending_send: CommanderStream<(CommandRequest,CommanderStream<Box<dyn ResponseType>>)>,
    integration: Rc<Box<dyn ChannelIntegration>>,
    channel: Channel,
    priority: PacketPriority,
    timeout: Option<f64>,
    messages: MessageSender
}

impl RequestQueueData {
    fn make_packet_sender(&self, packet: &RequestPacket) -> anyhow::Result<Pin<Box<dyn Future<Output=anyhow::Result<CborValue>>>>> {
        let channel = self.channel.clone();
        let priority = self.priority.clone();
        let integration = self.integration.clone();
        Ok(integration.get_sender(channel,priority,packet.serialize(&self.channel)?))
    }

    fn report<T>(&self, msg: anyhow::Result<T>) -> anyhow::Result<T> {
        if let Some(ref e) = msg.as_ref().err() {
            self.messages.send(DataMessage::PacketSendingError(self.channel.clone(),e.to_string()));
        }
        msg
    }

    fn set_timeout(&mut self, timeout: f64) {
        self.timeout = Some(timeout);
    }

    fn timeout(&self, streams: Vec<(Box<dyn ResponseType>,CommanderStream<Box<dyn ResponseType>>)>) {
        if let Some(timeout) = self.timeout {
            for (response,stream) in streams {
                let stream = stream.clone();
                let channel = self.channel.clone();
                let integration = self.integration.clone();
                let messages = self.messages.clone();
                cdr_add_timer(timeout, move || {
                    if stream.add_first(response) {
                        blackbox_log!(&format!("channel-{}",channel.to_string()),"timeout on channel '{}'",channel.to_string());
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
    pub fn new(commander: &PgCommander, receiver: &PayloadReceiverCollection, integration: &Rc<Box<dyn ChannelIntegration>>, channel: &Channel, priority: &PacketPriority, messages: &MessageSender) -> anyhow::Result<RequestQueue> {
        let out = RequestQueue(Arc::new(Mutex::new(RequestQueueData {
            receiver: receiver.clone(),
            builder: register_responses(),
            pending_send: CommanderStream::new(),
            integration: integration.clone(),
            channel: channel.clone(),
            priority: priority.clone(),
            timeout: None,
            messages: messages.clone()
        })));
        out.start(commander)?;
        Ok(out)
    }

    pub(crate) fn queue_command(&mut self, request: CommandRequest, stream: CommanderStream<Box<dyn ResponseType>>) {
        lock!(self.0).pending_send.add((request,stream));
    }

    fn start(&self, commander: &PgCommander) -> anyhow::Result<()> {
        let data = lock!(self.0);
        let name = format!("backend: '{}' {}",data.channel.to_string(),data.priority.to_string());
        drop(data);
        let self2 = self.clone();
        commander.add_task(PgCommanderTaskSpec {
            name,
            prio: 4,
            timeout: None,
            slot: None,
            task: Box::pin(self2.main_loop())
        });
        Ok(())
    }

    async fn build_packet(&self) -> anyhow::Result<(RequestPacket,HashMap<u64,CommanderStream<Box<dyn ResponseType>>>)> {
        let pending = lock!(self.0).pending_send.clone();
        let channel = lock!(self.0).channel.clone();
        let mut requests = pending.get_multi().await;
        let mut packet = RequestPacket::new();
        let mut channels = HashMap::new();
        let mut timeouts = vec![];
        for (r,c) in requests.drain(..) {
            blackbox_count!(&format!("channel-{}",channel.to_string()),"requests",1.);
            channels.insert(r.message_id(),c.clone());
            timeouts.push((r.request().to_failure(),c));
            packet.add(r);
        }
        lock!(self.0).timeout(timeouts);
        Ok((packet,channels))
    }

    async fn send_packet(&self, packet: &RequestPacket) -> anyhow::Result<ResponsePacket> {
        let channel = lock!(self.0).channel.clone();
        let sender = lock!(self.0).make_packet_sender(packet)?;
        blackbox_log!(&format!("channel-{}",channel.to_string()),"sending packet");
        blackbox_count!(&format!("channel-{}",channel.to_string()),"packets",1.);
        let response = sender.await?;
        blackbox_log!(&format!("channel-{}",channel.to_string()),"received response");
        let response = lock!(self.0).builder.new_packet(&response).context("Building response packet")?;
        Ok(response)
    }

    async fn send_or_fail_packet(&self, packet: &RequestPacket) -> ResponsePacket {
        let res = self.send_packet(packet).await;
        match lock!(self.0).report(res) {
            Ok(r) => r,
            Err(_) => packet.fail()
        }
    }

    async fn process_responses(&self, response: ResponsePacket, streams: &mut HashMap<u64,CommanderStream<Box<dyn ResponseType>>>) {
        let channel = lock!(self.0).channel.clone();
        let itn = lock!(self.0).integration.clone();
        let receiver = lock!(self.0).receiver.clone();
        let messages = lock!(self.0).messages.clone();
        let mut response = receiver.receive(&channel,response,&itn,&messages).await;
        for r in response.take_responses().drain(..) {
            let id = r.message_id();
            if let Some(stream) = streams.remove(&id) {
                blackbox_count!(&format!("channel-{}",channel.to_string()),"responses",1.);
                stream.add(r.into_response());
            }
        }
    }

    async fn process_request(&self, request: &mut RequestPacket, streams: &mut HashMap<u64,CommanderStream<Box<dyn ResponseType>>>) {
        let response = self.send_or_fail_packet(request).await;
        self.process_responses(response,streams).await;
    }

    fn err_context<T>(&self, a: anyhow::Result<T>, msg: &str) -> anyhow::Result<T> {
        a.with_context(|| format!("{} {}",msg,lock!(self.0).channel.to_string()))
    }

    pub(crate) fn set_timeout(&mut self, timeout: f64) {
        lock!(self.0).set_timeout(timeout);
    }

    async fn main_loop(self) -> anyhow::Result<()> {
        loop {
            let (mut request,mut streams) = self.err_context(self.build_packet().await,"preparing to send data")?;
            self.process_request(&mut request,&mut streams).await;
        }
    }
}
