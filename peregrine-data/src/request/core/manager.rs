use peregrine_toolkit::lock;
use commander::{ CommanderStream };
use peregrine_toolkit::sync::blocker::Blocker;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::{ Arc, Mutex };
use super::backoff::Backoff;
use super::queue::RequestQueue;
use super::request::{CommandRequest, RequestType};
use super::response::NewResponse;
use crate::core::channel::{Channel, ChannelIntegration, PacketPriority};
use crate::{PgCommanderTaskSpec, ResponsePacket, add_task};
use crate::api::MessageSender;
use crate::run::{ PgCommander };
use crate::util::message::DataMessage;

pub trait PayloadReceiver {
    fn receive(&self, channel: &Channel, response: ResponsePacket, itn: &Rc<Box<dyn ChannelIntegration>>, messages: &MessageSender) -> Pin<Box<dyn Future<Output=ResponsePacket>>>;
}

#[derive(Clone)]
pub struct PayloadReceiverCollection(Arc<Mutex<Vec<Rc<Box<dyn PayloadReceiver>>>>>);

impl PayloadReceiver for PayloadReceiverCollection {
    fn receive(&self, channel: &Channel, mut response: ResponsePacket, itn: &Rc<Box<dyn ChannelIntegration>>, messages: &MessageSender) ->  Pin<Box<dyn Future<Output=ResponsePacket>>> {
        let all = lock!(self.0).clone();
        let itn = itn.clone();
        let channel = channel.clone();
        let messages = messages.clone();
        Box::pin(async move {
            for receiver in all.iter() {
                response = receiver.receive(&channel,response,&itn,&messages).await;
            }
            response
        })
    }
}

enum QueueValue {
    Queue(RequestQueue),
    Redirect(Channel,PacketPriority)
}

pub struct RequestManagerData {
    integration: Rc<Box<dyn ChannelIntegration>>,
    receiver: PayloadReceiverCollection,
    commander: PgCommander,
    next_id: u64,
    queues: HashMap<(Channel,PacketPriority),QueueValue>,
    real_time_lock: Blocker,
    messages: MessageSender
}

impl RequestManagerData {
    pub fn new(integration: Box<dyn ChannelIntegration>, commander: &PgCommander, messages: &MessageSender) -> RequestManagerData {
        RequestManagerData {
            integration: Rc::new(integration),
            receiver: PayloadReceiverCollection(Arc::new(Mutex::new(vec![]))),
            commander: commander.clone(),
            next_id: 0,
            queues: HashMap::new(),
            real_time_lock: Blocker::new(),
            messages: messages.clone()
        }
    }

    pub fn message(&self, message: DataMessage) {
        self.messages.send(message);
    }

    pub fn add_receiver(&mut self, receiver: Box<dyn PayloadReceiver>) {
        lock!(self.receiver.0).push(Rc::new(receiver));
    }

    fn get_pace(&self, priority: &PacketPriority) -> &[f64] {
        match priority {
            PacketPriority::Batch => &[0.,5000.,10000.,20000.,20000.,20000.],
            PacketPriority::RealTime => &[0.,0.,500.,2000.,3000.,10000.]
        }
    }

    fn get_queue(&mut self, channel: &Channel, priority: &PacketPriority) -> Result<RequestQueue,DataMessage> {
        let mut channel = channel.clone();
        let mut priority = priority.clone();
        loop {
            let key = (channel.clone(),priority.clone());
            let missing = self.queues.get(&key).is_none();
            if missing {
                let commander = self.commander.clone();
                let integration = self.integration.clone(); // Rc why? XXX
                let mut queue = RequestQueue::new(&commander,&self.receiver,&integration,&channel,&priority,&self.messages,self.get_pace(&priority))?;
                match priority {
                    PacketPriority::RealTime => {
                        queue.set_realtime_block(&self.real_time_lock);
                    },
                    _ => {
                        queue.set_realtime_check(&self.real_time_lock);
                    }
                }
                self.queues.insert(key.clone(),QueueValue::Queue(queue));
            }
            match self.queues.get_mut(&key).unwrap() {
                QueueValue::Queue(q) => { 
                    return Ok(q.clone());
                },
                QueueValue::Redirect(new_channel,new_priority) => {
                    channel = new_channel.clone();
                    priority = new_priority.clone();
                }
            }
        }
    }

    fn execute(&mut self, channel: Channel, priority: PacketPriority, request: RequestType) -> Result<CommanderStream<NewResponse>,DataMessage> {
        let msg_id = self.next_id;
        self.next_id += 1;
        let request = CommandRequest::new(msg_id,request);
        let response_stream = CommanderStream::new();
        self.get_queue(&channel,&priority)?.queue_command(request,response_stream.clone());
        Ok(response_stream)
    }

    fn set_timeout(&mut self, channel: &Channel, priority: &PacketPriority, timeout: f64) -> anyhow::Result<()> {
        self.get_queue(channel,priority)?.set_timeout(timeout);
        self.integration.set_timeout(channel,timeout);
        Ok(())
    }

    fn set_lo_divert(&mut self, hi: &Channel, lo: &Channel) {
        if hi != lo {
            self.queues.insert((hi.clone(),PacketPriority::Batch),QueueValue::Redirect(lo.clone(),PacketPriority::Batch));
        }
    }
}

#[derive(Clone)]
pub struct RequestManager(Arc<Mutex<RequestManagerData>>);

impl RequestManager {
    pub fn new(integration: Box<dyn ChannelIntegration>, commander: &PgCommander, messages: &MessageSender) -> RequestManager {
        RequestManager(Arc::new(Mutex::new(RequestManagerData::new(integration,commander,messages))))
    }

    pub fn set_lo_divert(&self, channel_hi: &Channel, channel_lo: &Channel) {
        lock!(self.0).set_lo_divert(channel_hi,channel_lo);
    }

    pub fn set_timeout(&self, channel: &Channel, priority: &PacketPriority, timeout: f64) -> anyhow::Result<()> {
        lock!(self.0).set_timeout(channel,priority,timeout)
    }

    pub async fn execute(&mut self, channel: Channel, priority: PacketPriority, request: RequestType) -> Result<NewResponse,DataMessage> {
        let m = lock!(self.0).execute(channel,priority,request)?;
        Ok(m.get().await)
    }

    pub async fn submit<F,T>(&self, channel: &Channel, priority: &PacketPriority, request: RequestType, cb: F) 
                                                                    -> Result<T,DataMessage>
                                                                    where F: Fn(NewResponse) -> Result<T,String> {
        let mut backoff = Backoff::new(self,channel,priority);
        backoff.backoff(request,cb).await
    }

    pub fn execute_bactch(&self, channel: &Channel, request: RequestType) -> Result<(),DataMessage> {
        lock!(self.0).execute(channel.clone(),PacketPriority::Batch,request).map(|_| ())
    }

    pub(crate) fn execute_and_forget(&self, channel: &Channel, request: RequestType) {
        let commander = lock!(self.0).commander.clone();
        let mut manager = self.clone();
        let channel = channel.clone();
        add_task(&commander,PgCommanderTaskSpec {
            name: "message".to_string(),
            prio: 11,
            timeout: None,
            slot: None,
            task: Box::pin(async move { 
                manager.execute(channel,PacketPriority::Batch,request).await.ok();
                Ok(())
            }),
            stats: false
        });
    }

    pub fn add_receiver(&mut self, receiver: Box<dyn PayloadReceiver>) {
        lock!(self.0).add_receiver(receiver);
    }

    pub fn message(&self, message: DataMessage) {
        lock!(self.0).message(message);
    }
}
