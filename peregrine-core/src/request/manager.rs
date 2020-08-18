use async_trait::async_trait;
use commander::CommanderStream;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::rc::Rc;
use std::sync::{ Arc, Mutex };
use super::bootstrap::bootstrap_commands;
use super::program::program_commands;
use super::channel::{ Channel, PacketPriority };
use super::packet::{ RequestPacket, ResponsePacketBuilder, ResponsePacketBuilderBuilder };
use super::request::{ CommandRequest, CommandResponse, RequestType, ResponseType };
use crate::run::{ PgCommander, PgDauphin };
use crate::run::pgcommander::PgCommanderTaskSpec;
use serde_cbor::Value as CborValue;
use std::future::Future;
use std::pin::Pin;

fn register_responses() -> ResponsePacketBuilder {
    let mut rspbb = ResponsePacketBuilderBuilder::new();
    bootstrap_commands(&mut rspbb);
    program_commands(&mut rspbb);
    rspbb.build()
}

pub trait ChannelIntegration {
    fn get_sender(&self,channel: Channel, prio: PacketPriority, data: CborValue) -> Pin<Box<dyn Future<Output=anyhow::Result<CborValue>>>>;
}

struct RequestQueueData {
    dauphin: PgDauphin,
    builder: ResponsePacketBuilder,
    pending: CommanderStream<(CommandRequest,CommanderStream<Box<dyn ResponseType>>)>,
    integration: Rc<dyn ChannelIntegration>,
    channel: Channel,
    priority: PacketPriority
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

    async fn main_loop(self) -> anyhow::Result<()> {
        loop {
            /* build request */
            let pending = self.0.lock().unwrap().pending.clone();
            let mut requests = pending.get_multi().await;
            let mut packet = RequestPacket::new();
            let mut channels = HashMap::new();
            for (r,c) in requests.drain(..) {
                channels.insert(r.message_id(),c);
                packet.add(r);
            }
            /* send & receive */
            let data = self.0.lock().unwrap();
            let channel = data.channel.clone();
            let priority = data.priority.clone();
            let integration = data.integration.clone();
            drop(data);
            let response = integration.get_sender(channel,priority,packet.serialize()?).await?;
            /* process response */
            let data = self.0.lock().unwrap();
            let mut response = data.builder.new_packet(&response)?;
            data.dauphin.add_programs(&response)?;
            for r in response.take_responses().drain(..) {
                let id = r.message_id();
                if let Some(channel) = channels.get(&id) {
                    channel.add(r.into_response());
                }
            }
            drop(data);
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
}

#[derive(Clone)]
pub struct RequestManager(Arc<Mutex<RequestManagerData>>);

impl RequestManager {
    pub fn new<C>(integration: C, dauphin: &PgDauphin, commander: &PgCommander) -> RequestManager where C: ChannelIntegration+'static {
        RequestManager(Arc::new(Mutex::new(RequestManagerData::new(integration,dauphin,commander))))
    }

    pub async fn execute(&mut self, channel: Channel, priority: PacketPriority, request: Box<dyn RequestType>) -> anyhow::Result<Box<dyn ResponseType>> {
        Ok(self.0.lock().unwrap().execute(channel,priority,request)?.get().await)
    }
}