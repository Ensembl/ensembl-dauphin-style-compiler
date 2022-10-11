use peregrine_toolkit::{ lock };
use commander::{ CommanderStream };
use peregrine_toolkit::plumbing::oneshot::OneShot;
use peregrine_toolkit_async::sync::blocker::Blocker;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{ Arc, Mutex };
use super::attemptmatch::{AttemptMatch};
use super::backoff::Backoff;
use super::queue::{RequestQueue, QueueKey};
use super::request::MiniRequest;
use super::response::MiniResponseAttempt;
use super::sidecars::RequestSidecars;
use crate::core::channel::channelregistry::{ChannelRegistry};
use crate::core::channel::wrappedchannelsender::WrappedChannelSender;
use crate::core::version::VersionMetadata;
use crate::{PgCommanderTaskSpec, add_task, PacketPriority, BackendNamespace };
use crate::api::MessageSender;
use crate::run::{ PgCommander };
use crate::util::message::DataMessage;

#[derive(Clone)]
pub(crate) struct LowLevelRequestManager {
    version: VersionMetadata,
    sidecars: RequestSidecars,
    commander: PgCommander,
    shutdown: OneShot,
    matcher: AttemptMatch,
    queues: Arc<Mutex<HashMap<QueueKey,RequestQueue>>>,
    real_time_lock: Blocker,
    messages: MessageSender
}

impl LowLevelRequestManager {
    pub(crate) fn new(sidecars: &RequestSidecars, commander: &PgCommander, shutdown: &OneShot, messages: &MessageSender, version: &VersionMetadata) -> LowLevelRequestManager {
        LowLevelRequestManager {
            sidecars: sidecars.clone(),
            commander: commander.clone(),
            shutdown: shutdown.clone(),
            matcher: AttemptMatch::new(),
            queues: Arc::new(Mutex::new(HashMap::new())),
            real_time_lock: Blocker::new(),
            messages: messages.clone(),
            version: version.clone()
        }
    }

    pub fn message(&self, message: DataMessage) {
        self.messages.send(message);
    }

    fn create_queue(&self, key: &QueueKey) -> Result<RequestQueue,DataMessage> {
        let queue = RequestQueue::new(key,&self.commander,&self.real_time_lock,&self.matcher,&self.sidecars,&self.version,&self.messages)?;
        let queue2 = queue.clone();
        self.shutdown.add(move || {
            queue2.input_queue().close();
        });
        Ok(queue)
    }

    fn get_queue(&mut self, key: &QueueKey) -> Result<RequestQueue,DataMessage> {
        let mut queues = lock!(self.queues);
        let missing = queues.get(&key).is_none();
        if missing {
            let queue = self.create_queue(&key)?;
            queues.insert(key.clone(),queue);
        }
        Ok(queues.get_mut(&key).unwrap().clone()) // safe because of above insert
    }

    pub(crate) fn execute(&mut self, key: &QueueKey, request: &Rc<MiniRequest>) -> Result<CommanderStream<MiniResponseAttempt>,DataMessage> {
        let (request,stream) = self.matcher.make_attempt(request);
        self.get_queue(key)?.input_queue().add(request);
        Ok(stream.clone())
    }

    fn make_anon_key(&self, sender: &WrappedChannelSender, priority: &PacketPriority, name: &Option<BackendNamespace>) -> Result<QueueKey,DataMessage> {
        Ok(QueueKey::new(&sender,priority,&name))
    }

    pub(crate) async fn submit_direct<F,T>(&self, sender: &WrappedChannelSender, priority: &PacketPriority,  name: &Option<BackendNamespace>, request: &Rc<MiniRequest>, cb: F) 
                                                                    -> Result<T,DataMessage>
                                                                    where F: Fn(MiniResponseAttempt) -> Result<T,String> {
        let key = self.make_anon_key(sender,priority,name)?;
        let mut backoff = Backoff::new(self,&key);
        backoff.backoff(request,cb).await
    }
}

#[derive(Clone)]
pub struct RequestManager {
    low: LowLevelRequestManager,
    channel_registry: ChannelRegistry
}

impl RequestManager {
    pub(crate) fn new(low: &LowLevelRequestManager, channel_registry: &ChannelRegistry) -> RequestManager {
        RequestManager {
            low: low.clone(),
            channel_registry: channel_registry.clone()
        }
    }

    fn make_key(&self, name: &BackendNamespace, priority: &PacketPriority) -> Result<QueueKey,DataMessage> {
        let sender = self.channel_registry.name_to_sender(name)?;
        Ok(QueueKey::new(&sender,priority,&Some(name.clone())))
    }

    pub(crate) async fn submit<F,T>(&self, name: &BackendNamespace, priority: &PacketPriority, request: &Rc<MiniRequest>, cb: F) 
                                                                    -> Result<T,DataMessage>
                                                                    where F: Fn(MiniResponseAttempt) -> Result<T,String> {
        let key = self.make_key(name,priority)?;
        let mut backoff = Backoff::new(&self.low,&key);
        backoff.backoff(request,cb).await
    }

    pub(crate) async fn submit_direct<F,T>(&self, sender: &WrappedChannelSender, priority: &PacketPriority,  name: &Option<BackendNamespace>, request: MiniRequest, cb: F) 
                                                                    -> Result<T,DataMessage>
                                                                    where F: Fn(MiniResponseAttempt) -> Result<T,String> {
        self.low.submit_direct(sender, priority, name, &Rc::new(request), cb).await
    }

    pub(crate) fn execute_and_forget(&self, name: &BackendNamespace, request: MiniRequest) {
        let commander = self.low.commander.clone();
        let mut manager = self.clone();
        if let Ok(key) = self.make_key(name,&PacketPriority::Batch) {
            add_task(&commander,PgCommanderTaskSpec {
                name: "message".to_string(),
                prio: 11,
                timeout: None,
                slot: None,
                task: Box::pin(async move { 
                    manager.low.execute(&key,&Rc::new(request)).ok().unwrap().get().await;
                    Ok(())
                }),
                stats: false
            });
        }
    }

    pub fn message(&self, message: DataMessage) {
        self.low.message(message);
    }
}
