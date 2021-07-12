pub use std::sync::{ Arc, Mutex };
use std::future::Future;
use super::integrations::{ TestCommander, TestChannelIntegration, FakeDauphinReceiver };
use crate::{PeregrineCoreBase, api::MessageSender};
use crate::{ PgCommander, PgCommanderTaskSpec, PgDauphin, RequestManager };
use crate::{ ProgramLoader, StickStore, StickAuthorityStore, CountingPromise };
use crate::api::{ AgentStore, PeregrineApiQueue };
use crate::run::add_task;
use crate::switch::allotment::AllotmentPetitioner;
use crate::util::message::DataMessage;
use commander::Agent;
use peregrine_dauphin_queue::PgDauphinQueue;
use serde_cbor::Value as CborValue;
use url::Url;

#[derive(Clone)]
pub struct TestHelpers {
    pub base: PeregrineCoreBase,
    pub agent_store: AgentStore,
    pub console: Arc<Mutex<Vec<String>>>,
    pub channel: Box<TestChannelIntegration>,
    pub commander_inner: TestCommander,
    pub fdr: FakeDauphinReceiver
}

impl TestHelpers {
    pub(crate) fn new() -> TestHelpers {
        let mut agent_store = AgentStore::new();
        let console = Arc::new(Mutex::new(vec![]));
        let console2 = console.clone();
        let messages = MessageSender::new(move |msg| {
            console2.lock().unwrap().push(msg.to_string());
        });
        let booted = CountingPromise::new();
        let channel = Box::new(TestChannelIntegration::new());
        let commander_inner = TestCommander::new();
        let commander = PgCommander::new(Box::new(commander_inner.clone()));
        let manager = RequestManager::new(channel.clone(),&commander,&messages);
        let dauphin_queue = PgDauphinQueue::new();
        let dauphin = PgDauphin::new(&dauphin_queue).expect("d");
        let fdr = FakeDauphinReceiver::new(&commander,&dauphin_queue);
        let mut base = PeregrineCoreBase {
            messages,
            dauphin_queue,
            dauphin,
            commander,
            manager,
            booted,
            queue: PeregrineApiQueue::new(),
            allotment_petitioner: AllotmentPetitioner::new()
        };
        agent_store.set_program_loader(ProgramLoader::new(&base));
        agent_store.set_stick_authority_store(StickAuthorityStore::new(&base,&agent_store));
        base.manager.add_receiver(Box::new(base.dauphin.clone()));
        TestHelpers {
            console, base, agent_store,
            channel, commander_inner,
            fdr,
        }
    }
    
    pub(crate) fn run(&self, num: usize) {
        for _ in 0..num {
            self.commander_inner.tick();
        }
    }

    pub(crate) fn task<F>(&self, prog: F) where F: Future<Output=Result<(),DataMessage>> + 'static {
        add_task(&self.base.commander,PgCommanderTaskSpec {
            name: "program".to_string(),
            prio: 4,
            slot: None,
            timeout: None,
            task: Box::pin(prog),
            stats: false
        });
    }
}

pub(crate) fn urlc(idx: u32) -> Url {
    Url::parse(&(format!("http://a.com/{}",idx))).expect("b")
}

pub fn test_program() -> CborValue {
    let bytes = include_bytes!("test.dpb");
    serde_cbor::from_slice(bytes).expect("bad test program")
}