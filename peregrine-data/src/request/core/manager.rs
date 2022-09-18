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
use crate::core::channel::{Channel, ChannelIntegration, PacketPriority};
use crate::core::version::VersionMetadata;
use crate::{PgCommanderTaskSpec, add_task };
use crate::api::MessageSender;
use crate::run::{ PgCommander };
use crate::util::message::DataMessage;

enum QueueValue {
    Queue(RequestQueue),
    Redirect(Channel,PacketPriority)
}

pub struct RequestManagerData {
    integration: Rc<Box<dyn ChannelIntegration>>,
    version: VersionMetadata,
    sidecars: RequestSidecars,
    commander: PgCommander,
    shutdown: OneShot,
    matcher: AttemptMatch,
    queues: HashMap<(Channel,PacketPriority),QueueValue>,
    real_time_lock: Blocker,
    messages: MessageSender
}

impl RequestManagerData {
    pub(crate) fn new(integration: Box<dyn ChannelIntegration>, sidecars: &RequestSidecars, commander: &PgCommander, shutdown: &OneShot, messages: &MessageSender, version: &VersionMetadata) -> RequestManagerData {
        RequestManagerData {
            integration: Rc::new(integration),
            sidecars: sidecars.clone(),
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

    // pub fn add_receiver(&mut self, receiver: Box<dyn PayloadReceiver>) {
    //     lock!(self.receiver.0).push(Rc::new(receiver));
    // }

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

    fn create_queue(&self, sidecars: &RequestSidecars, channel: &Channel, priority: &PacketPriority) -> Result<RequestQueue,DataMessage> {
        let commander = self.commander.clone();
        let integration = self.integration.clone(); // Rc why? XXX
        let queue = RequestQueue::new(&commander,&self.real_time_lock,&self.matcher,sidecars,&integration,&self.version,&channel,&priority,&self.messages,self.get_pace(&priority),self.get_priority(&priority))?;
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
                self.queues.insert(key.clone(),QueueValue::Queue(self.create_queue(&self.sidecars,&channel,&priority)?));
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
        self.get_queue(&channel,&priority)?.input_queue().add(request);
        Ok(stream.clone())
    }

    fn set_timeout(&mut self, channel: &Channel, timeout: f64) -> anyhow::Result<()> {
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
    pub(crate) fn new(integration: Box<dyn ChannelIntegration>, sidecars: &RequestSidecars, commander: &PgCommander, shutdown: &OneShot, messages: &MessageSender, version: &VersionMetadata) -> NetworkRequestManager {
        NetworkRequestManager(Arc::new(Mutex::new(RequestManagerData::new(integration,sidecars,commander,shutdown,messages,version))))
    }

    pub fn set_supported_versions(&self, support: Option<&[u32]>, version: u32) {
        lock!(self.0).set_supported_versions(support,version);
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

    // pub fn add_receiver(&mut self, receiver: Box<dyn PayloadReceiver>) {
    //     lock!(self.0).add_receiver(receiver);
    // }

    pub fn message(&self, message: DataMessage) {
        lock!(self.0).message(message);
    }
}
