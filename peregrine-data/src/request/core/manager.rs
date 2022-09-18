use peregrine_toolkit::lock;
use commander::{ CommanderStream };
use peregrine_toolkit::plumbing::oneshot::OneShot;
use peregrine_toolkit_async::sync::blocker::Blocker;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::{ Arc, Mutex };
use super::attemptmatch::AttemptMatch;
use super::backoff::Backoff;
use super::queue::RequestQueue;
use super::request::BackendRequest;
use super::response::{BackendResponse};
use crate::core::channel::{Channel, ChannelIntegration, PacketPriority};
use crate::core::version::VersionMetadata;
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
    version: VersionMetadata,
    receiver: PayloadReceiverCollection,
    commander: PgCommander,
    shutdown: OneShot,
    matcher: AttemptMatch,
    queues: HashMap<(Channel,PacketPriority),QueueValue>,
    real_time_lock: Blocker,
    messages: MessageSender
}

impl RequestManagerData {
    pub fn new(integration: Box<dyn ChannelIntegration>, commander: &PgCommander, shutdown: &OneShot, messages: &MessageSender, version: &VersionMetadata) -> RequestManagerData {
        RequestManagerData {
            integration: Rc::new(integration),
            receiver: PayloadReceiverCollection(Arc::new(Mutex::new(vec![]))),
            commander: commander.clone(),
            shutdown: shutdown.clone(),
            matcher: AttemptMatch::new(),
            queues: HashMap::new(),
            real_time_lock: Blocker::new(),
            messages: messages.clone(),
            version: version.clone()
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

    fn get_priority(&self, priority: &PacketPriority) -> u8 {
        match priority {
            PacketPriority::Batch => 5,
            PacketPriority::RealTime => 3
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
                let mut queue = RequestQueue::new(&commander,&self.matcher,&self.receiver,&integration,&self.version,&channel,&priority,&self.messages,self.get_pace(&priority),self.get_priority(&priority))?;
                match priority {
                    PacketPriority::RealTime => {
                        queue.set_realtime_block(&self.real_time_lock);
                    },
                    _ => {
                        queue.set_realtime_check(&self.real_time_lock);
                    }
                }
                let mut queue2 = queue.clone();
                self.shutdown.add(move || {
                    queue2.shutdown();
                });
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

    fn execute(&mut self, channel: Channel, priority: PacketPriority, request: &BackendRequest) -> Result<CommanderStream<BackendResponse>,DataMessage> {
        let (request,stream) = self.matcher.make_attempt(request);
        self.get_queue(&channel,&priority)?.queue_command(request);
        Ok(stream.clone())
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

    fn set_supported_versions(&self, supports: Option<&[u32]>, version: u32) {
        self.integration.set_supported_versions(supports,version);
    }
}

#[derive(Clone)]
pub struct NetworkRequestManager(Arc<Mutex<RequestManagerData>>);

impl NetworkRequestManager {
    pub fn new(integration: Box<dyn ChannelIntegration>, commander: &PgCommander, shutdown: &OneShot, messages: &MessageSender, version: &VersionMetadata) -> NetworkRequestManager {
        NetworkRequestManager(Arc::new(Mutex::new(RequestManagerData::new(integration,commander,shutdown,messages,version))))
    }

    pub fn set_supported_versions(&self, support: Option<&[u32]>, version: u32) {
        lock!(self.0).set_supported_versions(support,version);
    }

    pub fn set_lo_divert(&self, channel_hi: &Channel, channel_lo: &Channel) {
        lock!(self.0).set_lo_divert(channel_hi,channel_lo);
    }

    pub fn set_timeout(&self, channel: &Channel, priority: &PacketPriority, timeout: f64) -> anyhow::Result<()> {
        lock!(self.0).set_timeout(channel,priority,timeout)
    }

    pub(crate) async fn execute(&mut self, channel: Channel, priority: PacketPriority, request: &BackendRequest) -> Result<BackendResponse,DataMessage> {
        let m = lock!(self.0).execute(channel,priority,request)?;
        Ok(m.get().await)
    }

    pub(crate) async fn submit<F,T>(&self, channel: &Channel, priority: &PacketPriority, request: &BackendRequest, cb: F) 
                                                                    -> Result<T,DataMessage>
                                                                    where F: Fn(BackendResponse) -> Result<T,String> {
        let mut backoff = Backoff::new(self,channel,priority);
        backoff.backoff(request,cb).await
    }

    pub(crate) fn execute_and_forget(&self, channel: &Channel, request: BackendRequest) {
        let commander = lock!(self.0).commander.clone();
        let mut manager = self.clone();
        let channel = channel.clone();
        add_task(&commander,PgCommanderTaskSpec {
            name: "message".to_string(),
            prio: 11,
            timeout: None,
            slot: None,
            task: Box::pin(async move { 
                manager.execute(channel,PacketPriority::Batch,&request).await.ok();
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
