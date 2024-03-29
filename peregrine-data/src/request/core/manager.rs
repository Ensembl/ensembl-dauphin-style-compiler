use peregrine_toolkit::error::Error;
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
use super::minirequest::MiniRequest;
use super::miniresponse::{MiniResponseAttempt, MiniResponseError};
use super::sidecars::RequestSidecars;
use crate::core::channel::channelregistry::{ChannelRegistry};
use crate::core::channel::wrappedchannelsender::WrappedChannelSender;
use crate::core::version::VersionMetadata;
use crate::{PgCommanderTaskSpec, add_task, PacketPriority, BackendNamespace, ChannelSender };
use crate::api::MessageSender;
use crate::run::{ PgCommander };

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

    pub fn message(&self, message: Error) {
        self.messages.send(message);
    }

    fn create_queue(&self, key: &QueueKey) -> Result<RequestQueue,Error> {
        let queue = RequestQueue::new(key,&self.commander,&self.real_time_lock,&self.matcher,&self.sidecars,&self.version,&self.messages)?;
        let queue2 = queue.clone();
        self.shutdown.add(move || {
            queue2.input_queue().close();
        });
        Ok(queue)
    }

    fn get_queue(&mut self, key: &QueueKey) -> Result<RequestQueue,Error> {
        let mut queues = lock!(self.queues);
        let missing = queues.get(&key).is_none();
        if missing {
            let queue = self.create_queue(&key)?;
            queues.insert(key.clone(),queue);
        }
        Ok(queues.get_mut(&key).unwrap().clone()) // safe because of above insert
    }

    pub(crate) fn execute(&mut self, key: &QueueKey, request: &Rc<MiniRequest>) -> Result<CommanderStream<MiniResponseAttempt>,Error> {
        let (request,stream) = self.matcher.make_attempt(request);
        self.get_queue(key)?.input_queue().add(request);
        Ok(stream.clone())
    }

    fn make_anon_key(&self, sender: &WrappedChannelSender, priority: &PacketPriority, name: &Option<BackendNamespace>) -> Result<QueueKey,Error> {
        Ok(QueueKey::new(&sender,priority,&name))
    }

    pub(crate) async fn submit_direct<F,T>(&self, sender: &WrappedChannelSender, priority: &PacketPriority, name: &Option<BackendNamespace>, request: &Rc<MiniRequest>, cb: F) 
                                                                    -> Result<T,Error>
            where F: Fn(MiniResponseAttempt) -> Result<T,MiniResponseError> {
        let key = self.make_anon_key(sender,priority,name)?;
        let repeats = if sender.backoff() { priority.repeats() } else { 1 };
        let mut backoff = Backoff::new(self,&key,repeats);
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

    fn make_key(&self, name: &BackendNamespace, priority: &PacketPriority) -> Result<(QueueKey,bool),Error> {
        let sender = self.channel_registry.name_to_sender(name)?;
        Ok((QueueKey::new(&sender,priority,&Some(name.clone())),sender.backoff()))
    }

    pub(crate) async fn submit<F,T>(&self, name: &BackendNamespace, priority: &PacketPriority, request: &Rc<MiniRequest>, cb: F) 
                                                                    -> Result<T,Error>
            where F: Fn(MiniResponseAttempt) -> Result<T,MiniResponseError> {
        let (key,enable_backoff) = self.make_key(name,priority)?;
        let repeats = if enable_backoff { priority.repeats() } else { 1 };
        let mut backoff = Backoff::new(&self.low,&key,repeats);
        backoff.backoff(request,cb).await
    }

    pub(crate) async fn submit_direct<F,T>(&self, sender: &WrappedChannelSender, priority: &PacketPriority, name: &Option<BackendNamespace>, request: MiniRequest, cb: F) 
                                                                    -> Result<T,Error>
            where F: Fn(MiniResponseAttempt) -> Result<T,MiniResponseError> {
        self.low.submit_direct(sender,priority,name,&Rc::new(request),cb).await
    }

    pub(crate) fn execute_and_forget(&self, name: &BackendNamespace, request: MiniRequest) {
        let commander = self.low.commander.clone();
        let mut manager = self.clone();
        if let Ok((key,_)) = self.make_key(name,&PacketPriority::Batch) {
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

    pub fn message(&self, message: Error) {
        self.low.message(message);
    }
}
