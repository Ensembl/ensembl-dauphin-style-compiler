use commander::{ CommanderStream };
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::rc::Rc;
use std::sync::{ Arc, Mutex };
use super::channel::{ Channel, PacketPriority, ChannelIntegration };
use super::queue::RequestQueue;
use super::request::{ CommandRequest, RequestType, ResponseType };
use crate::run::{ PgCommander, PgDauphin };

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

    fn get_queue(&mut self, channel: &Channel, priority: &PacketPriority) -> anyhow::Result<&mut RequestQueue> {
        Ok(match self.queues.entry((channel.clone(),priority.clone())) {
            Entry::Vacant(e) => { 
                let commander = self.commander.clone();
                let integration = self.integration.clone();
                e.insert(RequestQueue::new(&commander,&self.dauphin,&integration,&channel,&priority)?)
            },
            Entry::Occupied(e) => { e.into_mut() }
        })
    }

    pub fn execute(&mut self, channel: Channel, priority: PacketPriority, request: Box<dyn RequestType>) -> anyhow::Result<CommanderStream<Box<dyn ResponseType>>> {
        let msg_id = self.next_id;
        self.next_id += 1;
        let request = CommandRequest::new(msg_id,request);
        let response_channel = CommanderStream::new();
        self.get_queue(&channel,&priority)?.queue_command(request,response_channel.clone());
        Ok(response_channel)
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
    pub fn new<C>(integration: C, dauphin: &PgDauphin, commander: &PgCommander) -> RequestManager where C: ChannelIntegration+'static {
        RequestManager(Arc::new(Mutex::new(RequestManagerData::new(integration,dauphin,commander))))
    }

    pub fn set_timeout(&self, channel: &Channel, priority: &PacketPriority, timeout: f64) -> anyhow::Result<()> {
        self.0.lock().unwrap().set_timeout(channel,priority,timeout)
    }

    pub async fn execute(&mut self, channel: Channel, priority: PacketPriority, request: Box<dyn RequestType>) -> anyhow::Result<Box<dyn ResponseType>> {
        Ok(self.0.lock().unwrap().execute(channel,priority,request)?.get().await)
    }

    pub fn error(&self, channel: &Channel, msg: &str) {
        self.0.lock().unwrap().error(channel,msg);
    }
}
