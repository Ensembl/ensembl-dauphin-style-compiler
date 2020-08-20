use anyhow::Context;
use blackbox::{ blackbox_time, blackbox_count, blackbox_log };
use commander::CommanderStream;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::{ Arc, Mutex };
use super::bootstrap::bootstrap_commands;
use super::program::program_commands;
use super::channel::{ Channel, PacketPriority, ChannelIntegration };
use super::packet::{ RequestPacket, ResponsePacket, ResponsePacketBuilder, ResponsePacketBuilderBuilder };
use super::request::{ CommandRequest, CommandResponse, RequestType, ResponseType };
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
    priority: PacketPriority
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

    fn process_responses(&self, response: &mut ResponsePacket, channels: &mut HashMap<u64,CommanderStream<Box<dyn ResponseType>>>) {
        let channel = self.channel.clone();
        self.add_programs(&channel,response);
        for r in response.take_responses().drain(..) {
            let id = r.message_id();
            if let Some(channel) = channels.remove(&id) {
                blackbox_count!(&format!("channel-",channel.to_string()),"responses",1);
                channel.add(r.into_response());
            }
        }
    }
}

#[derive(Clone)]
struct RequestQueue(Arc<Mutex<RequestQueueData>>);

impl RequestQueue {
    pub fn new(commander: &PgCommander, dauphin: &PgDauphin, integration: &Rc<dyn ChannelIntegration>, channel: &Channel, priority: PacketPriority) -> anyhow::Result<RequestQueue> {
        let out = RequestQueue(Arc::new(Mutex::new(RequestQueueData {
            dauphin: dauphin.clone(),
            builder: register_responses(),
            pending: CommanderStream::new(),
            integration: integration.clone(),
            channel: channel.clone(),
            priority
        })));
        out.start(commander)?;
        Ok(out)
    }

    fn queue_command(&mut self, request: CommandRequest, channel: CommanderStream<Box<dyn ResponseType>>) {
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
        for (r,c) in requests.drain(..) {
            blackbox_count!(&format!("channel-",channel.to_string()),"requests",1);
            channels.insert(r.message_id(),c);
            packet.add(r);
        }
        Ok((packet,channels))
    }

    async fn send_packet(&self, packet: &RequestPacket) -> anyhow::Result<ResponsePacket> {
        let sender = self.0.lock().unwrap().make_packet_sender(packet)?;
        blackbox_log!(&format!("channel-{}",self.channel.to_string()),"sending packet");
        blackbox_count!(&format!("channel-",channel.to_string()),"packets",1);
        let response = blackbox_time!(&format!("channel-",channel.to_string()),"roundtrip",{
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

    async fn main_loop(self) -> anyhow::Result<()> {
        loop {
            let (mut request,mut channels) = self.err_context(self.build_packet().await,"preparing to send data")?;
            self.process_request(&mut request,&mut channels).await;
        }
    }
}

pub struct RequestManagerData {
    integration: Rc<dyn ChannelIntegration>,
    dauphin: PgDauphin,
    commander: PgCommander,
    next_id: u64,
    queues: HashMap<(Channel,PacketPriority),RequestQueue>
}

impl RequestManagerData {
    pub fn new<C>(integration: C, dauphin: &PgDauphin, commander: &PgCommander) -> RequestManagerData where C: ChannelIntegration+'static {
        RequestManagerData {
            integration: Rc::new(integration),
            dauphin: dauphin.clone(),
            commander: commander.clone(),
            next_id: 0,
            queues: HashMap::new()
        }
    }

    fn error(&self, channel: &Channel, msg: &str) {
        self.integration.error(channel,msg);
    }

    pub fn execute(&mut self, channel: Channel, priority: PacketPriority, request: Box<dyn RequestType>) -> anyhow::Result<CommanderStream<Box<dyn ResponseType>>> {
        let msg_id = self.next_id;
        self.next_id += 1;
        let request = CommandRequest::new(msg_id,request);
        let response_channel = CommanderStream::new();
        match self.queues.entry((channel.clone(),priority.clone())) {
            Entry::Vacant(e) => { 
                let commander = self.commander.clone();
                let integration = self.integration.clone();
                e.insert(RequestQueue::new(&commander,&self.dauphin,&integration,&channel,priority)?)
            },
            Entry::Occupied(e) => { e.into_mut() }
        }.queue_command(request,response_channel.clone());
        Ok(response_channel)
    }

    pub fn set_timeout(&self, channel: &Channel, timeout: f64) {
        self.integration.set_timeout(channel,timeout);
    }
}

#[derive(Clone)]
pub struct RequestManager(Arc<Mutex<RequestManagerData>>);

impl RequestManager {
    pub fn new<C>(integration: C, dauphin: &PgDauphin, commander: &PgCommander) -> RequestManager where C: ChannelIntegration+'static {
        RequestManager(Arc::new(Mutex::new(RequestManagerData::new(integration,dauphin,commander))))
    }

    pub fn set_timeout(&self, channel: &Channel, timeout: f64) {
        self.0.lock().unwrap().set_timeout(channel,timeout);
    }

    pub async fn execute(&mut self, channel: Channel, priority: PacketPriority, request: Box<dyn RequestType>) -> anyhow::Result<Box<dyn ResponseType>> {
        Ok(self.0.lock().unwrap().execute(channel,priority,request)?.get().await)
    }

    pub fn error(&self, channel: &Channel, msg: &str) {
        self.0.lock().unwrap().error(channel,msg);
    }
}
