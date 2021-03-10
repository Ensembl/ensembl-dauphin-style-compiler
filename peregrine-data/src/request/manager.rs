use crate::lock;
use anyhow::bail;
use commander::{ CommanderStream };
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::{ Arc, Mutex };
use super::channel::{ Channel, PacketPriority, ChannelIntegration };
use super::queue::RequestQueue;
use super::packet::ResponsePacket;
use super::request::{ CommandRequest, RequestType, ResponseType };
use crate::run::{ PgCommander, PgDauphin };

pub trait PayloadReceiver {
    fn receive(&self, channel: &Channel, response: ResponsePacket, itn: &Rc<Box<dyn ChannelIntegration>>) -> Pin<Box<dyn Future<Output=ResponsePacket>>>;
}

#[derive(Clone)]
pub struct PayloadReceiverCollection(Arc<Mutex<Vec<Rc<Box<dyn PayloadReceiver>>>>>);

impl PayloadReceiver for PayloadReceiverCollection {
    fn receive(&self, channel: &Channel, mut response: ResponsePacket, itn: &Rc<Box<dyn ChannelIntegration>>) ->  Pin<Box<dyn Future<Output=ResponsePacket>>> {
        let all = lock!(self.0).clone();
        let itn = itn.clone();
        let channel = channel.clone();
        Box::pin(async move {
            for receiver in all.iter() {
                response = receiver.receive(&channel,response,&itn).await;
            }
            response
        })
    }
}

pub struct RequestManagerData {
    integration: Rc<Box<dyn ChannelIntegration>>,
    receiver: PayloadReceiverCollection,
    commander: PgCommander,
    next_id: u64,
    queues: HashMap<(Channel,PacketPriority),RequestQueue>
}

impl RequestManagerData {
    pub fn new(integration: Box<dyn ChannelIntegration>, commander: &PgCommander) -> RequestManagerData {
        RequestManagerData {
            integration: Rc::new(integration),
            receiver: PayloadReceiverCollection(Arc::new(Mutex::new(vec![]))),
            commander: commander.clone(),
            next_id: 0,
            queues: HashMap::new()
        }
    }

    pub fn add_receiver(&mut self, receiver: Box<dyn PayloadReceiver>) {
        lock!(self.receiver.0).push(Rc::new(receiver));
    }

    fn error(&self, channel: &Channel, msg: &str) {
        self.integration.error(channel,msg);
    }

    fn warn(&self, channel: &Channel, msg: &str) {
        self.integration.warn(channel,msg);
    }

    fn get_queue(&mut self, channel: &Channel, priority: &PacketPriority) -> anyhow::Result<&mut RequestQueue> {
        Ok(match self.queues.entry((channel.clone(),priority.clone())) {
            Entry::Vacant(e) => { 
                let commander = self.commander.clone();
                let integration = self.integration.clone(); // Rc why? XXX
                e.insert(RequestQueue::new(&commander,&self.receiver,&integration,&channel,&priority)?)
            },
            Entry::Occupied(e) => { e.into_mut() }
        })
    }

    pub fn execute(&mut self, channel: Channel, priority: PacketPriority, request: Box<dyn RequestType>) -> anyhow::Result<CommanderStream<Box<dyn ResponseType>>> {
        let msg_id = self.next_id;
        self.next_id += 1;
        let request = CommandRequest::new(msg_id,request);
        let response_stream = CommanderStream::new();
        self.get_queue(&channel,&priority)?.queue_command(request,response_stream.clone());
        Ok(response_stream)
    }

    pub fn set_timeout(&mut self, channel: &Channel, priority: &PacketPriority, timeout: f64) -> anyhow::Result<()> {
        self.get_queue(channel,priority)?.set_timeout(timeout);
        self.integration.set_timeout(channel,timeout);
        Ok(())
    }
}

#[derive(Clone)]
pub struct RequestManager(Arc<Mutex<RequestManagerData>>);

impl RequestManager {
    pub fn new(integration: Box<dyn ChannelIntegration>, commander: &PgCommander) -> RequestManager {
        RequestManager(Arc::new(Mutex::new(RequestManagerData::new(integration,commander))))
    }

    pub fn set_timeout(&self, channel: &Channel, priority: &PacketPriority, timeout: f64) -> anyhow::Result<()> {
        lock!(self.0).set_timeout(channel,priority,timeout)
    }

    pub async fn execute(&mut self, channel: Channel, priority: PacketPriority, request: Box<dyn RequestType>) -> anyhow::Result<Box<dyn ResponseType>> {
        let m = lock!(self.0).execute(channel,priority,request)?;
        let out = Ok(m.get().await);
        out
    }

    pub fn execute_background(&self, channel: &Channel, request: Box<dyn RequestType>) -> anyhow::Result<()> {
        lock!(self.0).execute(channel.clone(),PacketPriority::Batch,request).map(|_| ())
    }

    pub fn add_receiver(&mut self, receiver: Box<dyn PayloadReceiver>) {
        lock!(self.0).add_receiver(receiver);
    }

    pub fn error(&self, channel: &Channel, msg: &str) {
        lock!(self.0).error(channel,msg);
    }

    pub fn warn(&self, channel: &Channel, msg: &str) {
        lock!(self.0).warn(channel,msg);
    }
}
