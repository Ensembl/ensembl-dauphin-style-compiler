use peregrine_toolkit::lock;
use commander::{ CommanderStream };
use peregrine_toolkit::plumbing::oneshot::OneShot;
use peregrine_toolkit_async::sync::blocker::Blocker;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{ Arc, Mutex };
use super::attemptmatch::AttemptMatch;
use super::backoff::Backoff;
use super::queue::RequestQueue;
use super::request::BackendRequest;
use super::response::{BackendResponse};
use super::sidecars::RequestSidecars;
use crate::core::version::VersionMetadata;
use crate::{PgCommanderTaskSpec, add_task, ChannelSender, ChannelIntegration, Channel, PacketPriority };
use crate::api::MessageSender;
use crate::run::{ PgCommander };
use crate::util::message::DataMessage;

pub struct ChannelRegistry {
    integrations: Vec<Rc<dyn ChannelIntegration>>,
    channels: HashMap<Channel,Arc<dyn ChannelSender>>
}

impl ChannelRegistry {
    fn new() -> ChannelRegistry {
        ChannelRegistry {
            integrations: vec![],
            channels: HashMap::new()
        }
    }

    fn add(&mut self, integration: Rc<dyn ChannelIntegration>) {
        self.integrations.push(integration);
    }

    fn lookup(&mut self, channel: &Channel) -> Result<&Arc<dyn ChannelSender>,DataMessage> {
        if !self.channels.contains_key(channel) {
            for integration in &self.integrations {
                if let Some(sender) = integration.make_sender(channel) {
                    self.channels.insert(channel.clone(),sender);
                    break;
                }
            }
        }
        self.channels.get(channel).ok_or_else(|| DataMessage::BackendRefused(channel.clone(),"no such integration".to_string()))
    }
}

enum QueueValue {
    Queue(RequestQueue),
    Redirect(Channel,PacketPriority)
}

pub struct RequestManagerData {
    version: VersionMetadata,
    sidecars: RequestSidecars,
    commander: PgCommander,
    shutdown: OneShot,
    matcher: AttemptMatch,
    channel_registry: ChannelRegistry,
    queues: HashMap<(Channel,PacketPriority),QueueValue>,
    real_time_lock: Blocker,
    messages: MessageSender
}

impl RequestManagerData {
    pub(crate) fn new(sidecars: &RequestSidecars, commander: &PgCommander, shutdown: &OneShot, messages: &MessageSender, version: &VersionMetadata) -> RequestManagerData {
        RequestManagerData {
            sidecars: sidecars.clone(),
            commander: commander.clone(),
            shutdown: shutdown.clone(),
            matcher: AttemptMatch::new(),
            queues: HashMap::new(),
            real_time_lock: Blocker::new(),
            messages: messages.clone(),
            version: version.clone(),
            channel_registry: ChannelRegistry::new()
        }
    }

    pub fn message(&self, message: DataMessage) {
        self.messages.send(message);
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

    fn create_queue(&mut self, channel: &Channel, priority: &PacketPriority) -> Result<RequestQueue,DataMessage> {
        let commander = self.commander.clone();
        let integration = self.channel_registry.lookup(channel)?.clone();
        let queue = RequestQueue::new(&commander,&self.real_time_lock,&self.matcher,&self.sidecars,&integration,&self.version,&channel,&priority,&self.messages,self.get_pace(&priority),self.get_priority(&priority))?;
        let queue2 = queue.clone();
        self.shutdown.add(move || {
            queue2.input_queue().close();
        });
        Ok(queue)
    }

    fn get_queue(&mut self, channel: &Channel, priority: &PacketPriority) -> Result<RequestQueue,DataMessage> {
        let mut channel = channel.clone();
        let mut priority = priority.clone();
        loop {
            let key = (channel.clone(),priority.clone());
            let missing = self.queues.get(&key).is_none();
            if missing {
                let queue = self.create_queue(&channel,&priority)?;
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

    fn add_channel_integration(&mut self, channel: Rc<dyn ChannelIntegration>) {
        self.channel_registry.add(channel);
    }

    fn execute(&mut self, channel: Channel, priority: PacketPriority, request: &BackendRequest) -> Result<CommanderStream<BackendResponse>,DataMessage> {
        let (request,stream) = self.matcher.make_attempt(request);
        self.get_queue(&channel,&priority)?.input_queue().add(request);
        Ok(stream.clone())
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
    pub(crate) fn new(sidecars: &RequestSidecars, commander: &PgCommander, shutdown: &OneShot, messages: &MessageSender, version: &VersionMetadata) -> RequestManager {
        RequestManager(Arc::new(Mutex::new(RequestManagerData::new(sidecars,commander,shutdown,messages,version))))
    }

    pub fn add_channel_integration(&mut self, channel: Rc<dyn ChannelIntegration>) {
        lock!(self.0).add_channel_integration(channel);
    }

    pub fn set_lo_divert(&self, channel_hi: &Channel, channel_lo: &Channel) {
        lock!(self.0).set_lo_divert(channel_hi,channel_lo);
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

    pub fn message(&self, message: DataMessage) {
        lock!(self.0).message(message);
    }
}
