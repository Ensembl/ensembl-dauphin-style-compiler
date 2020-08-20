use anyhow::Context;
use blackbox::{ blackbox_time, blackbox_count, blackbox_log };
use commander::{ CommanderStream, cdr_add_timer };
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::{ Arc, Mutex };
use super::bootstrap::bootstrap_commands;
use super::program::program_commands;
use super::channel::{ Channel, PacketPriority, ChannelIntegration };
use super::packet::{ RequestPacket, ResponsePacket, ResponsePacketBuilder, ResponsePacketBuilderBuilder };
use super::request::{ CommandRequest, ResponseType };
use crate::run::{ PgCommander, PgDauphin };
use crate::run::pgcommander::PgCommanderTaskSpec;
use serde_cbor::Value as CborValue;

fn register_responses() -> ResponsePacketBuilder {
    let mut rspbb = ResponsePacketBuilderBuilder::new();
    bootstrap_commands(&mut rspbb);
    program_commands(&mut rspbb);
    rspbb.build()
}

struct RequestQueueData {
    dauphin: PgDauphin,
    builder: ResponsePacketBuilder,
    pending: CommanderStream<(CommandRequest,CommanderStream<Box<dyn ResponseType>>)>,
    integration: Rc<dyn ChannelIntegration>,
    channel: Channel,
    priority: PacketPriority,
    timeout: Option<f64>
}

impl RequestQueueData {
    fn make_packet_sender(&self, packet: &RequestPacket) -> anyhow::Result<Pin<Box<dyn Future<Output=anyhow::Result<CborValue>>>>> {
        let channel = self.channel.clone();
        let priority = self.priority.clone();
        let integration = self.integration.clone();
        Ok(integration.get_sender(channel,priority,packet.serialize()?))
    }

    fn report<T>(&self, msg: anyhow::Result<T>) -> anyhow::Result<T> {
        if let Some(ref e) = msg.as_ref().err() {
            self.integration.error(&self.channel,&format!("error: {}",e));
        }
        msg
    }

    fn add_programs(&self, channel: &Channel, response: &ResponsePacket) {
        for bundle in response.programs().clone().iter() {
            match self.report(self.dauphin.add_binary(channel,bundle.bundle_name(),bundle.program())) {
                Ok(_) => {
                    for (in_channel_name,in_bundle_name) in bundle.name_map() {
                        self.dauphin.register(channel,in_channel_name,bundle.bundle_name(),in_bundle_name);
                    }
                },
                Err(_) => {
                    for (in_channel_name,_) in bundle.name_map() {
                        self.dauphin.mark_missing(channel,in_channel_name);
                    }
                }
            }
        }
    }

    fn process_responses(&self, response: &mut ResponsePacket, streams: &mut HashMap<u64,CommanderStream<Box<dyn ResponseType>>>) {
        let channel = self.channel.clone();
        self.add_programs(&channel,response);
        for r in response.take_responses().drain(..) {
            let id = r.message_id();
            if let Some(stream) = streams.remove(&id) {
                blackbox_count!(&format!("channel-",self.channel.to_string()),"responses",1);
                stream.add(r.into_response());
            }
        }
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
                cdr_add_timer(timeout, move || {
                    if stream.add_first(response) {
                        blackbox_log!(&format!("channel-",channel.to_string()),"timeout on channel '{}'",channel.to_string());
                        integration.error(&channel,&format!("timeout on channel '{}'",channel.to_string()));
                    }
                });
            }
        }
    }
}

#[derive(Clone)]
pub struct RequestQueue(Arc<Mutex<RequestQueueData>>);

impl RequestQueue {
    pub fn new(commander: &PgCommander, dauphin: &PgDauphin, integration: &Rc<dyn ChannelIntegration>, channel: &Channel, priority: &PacketPriority) -> anyhow::Result<RequestQueue> {
        let out = RequestQueue(Arc::new(Mutex::new(RequestQueueData {
            dauphin: dauphin.clone(),
            builder: register_responses(),
            pending: CommanderStream::new(),
            integration: integration.clone(),
            channel: channel.clone(),
            priority: priority.clone(),
            timeout: None
        })));
        out.start(commander)?;
        Ok(out)
    }

    pub(crate) fn queue_command(&mut self, request: CommandRequest, channel: CommanderStream<Box<dyn ResponseType>>) {
        self.0.lock().unwrap().pending.add((request,channel));
    }

    fn start(&self, commander: &PgCommander) -> anyhow::Result<()> {
        let data = self.0.lock().unwrap();
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
        let pending = self.0.lock().unwrap().pending.clone();
        let mut requests = pending.get_multi().await;
        let mut packet = RequestPacket::new();
        let mut channels = HashMap::new();
        let mut timeouts = vec![];
        for (r,c) in requests.drain(..) {
            blackbox_count!(&format!("channel-",self.channel.to_string()),"requests",1);
            channels.insert(r.message_id(),c.clone());
            timeouts.push((r.request().to_failure(),c));
            packet.add(r);
        }
        self.0.lock().unwrap().timeout(timeouts);
        Ok((packet,channels))
    }

    async fn send_packet(&self, packet: &RequestPacket) -> anyhow::Result<ResponsePacket> {
        let sender = self.0.lock().unwrap().make_packet_sender(packet)?;
        blackbox_log!(&format!("channel-{}",self.channel.to_string()),"sending packet");
        blackbox_count!(&format!("channel-",self.channel.to_string()),"packets",1);
        let response = blackbox_time!(&format!("channel-",self.channel.to_string()),"roundtrip",{
            sender.await?
        });
        blackbox_log!(&format!("channel-{}",self.channel.to_string()),"received response");
        let response = self.0.lock().unwrap().builder.new_packet(&response)?;
        Ok(response)
    }

    async fn send_or_fail_packet(&self, packet: &RequestPacket) -> ResponsePacket {
        let res = self.send_packet(packet).await;
        match self.0.lock().unwrap().report(res) {
            Ok(r) => r,
            Err(_) => packet.fail()
        }
    }

    async fn process_request(&self, request: &mut RequestPacket, channels: &mut HashMap<u64,CommanderStream<Box<dyn ResponseType>>>) {
        let mut response = self.send_or_fail_packet(request).await;
        self.0.lock().unwrap().process_responses(&mut response,channels);
    }

    fn err_context<T>(&self, a: anyhow::Result<T>, msg: &str) -> anyhow::Result<T> {
        a.with_context(|| format!("{} {}",msg,self.0.lock().unwrap().channel.to_string()))
    }

    pub(crate) fn set_timeout(&mut self, timeout: f64) {
        self.0.lock().unwrap().set_timeout(timeout);
    }

    async fn main_loop(self) -> anyhow::Result<()> {
        loop {
            let (mut request,mut channels) = self.err_context(self.build_packet().await,"preparing to send data")?;
            self.process_request(&mut request,&mut channels).await;
        }
    }
}
