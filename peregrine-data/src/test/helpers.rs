use std::future::Future;
use super::integrations::{ TestCommander, TestChannelIntegration, TestConsole, FakeDauphinReceiver };
use crate::api::MessageSender;
use crate::{ PgCommander, PgCommanderTaskSpec, PgDauphin, RequestManager };
use crate::{ ProgramLoader, StickStore, StickAuthorityStore, CountingPromise };
use crate::run::add_task;
use crate::util::message::DataMessage;
use peregrine_dauphin_queue::PgDauphinQueue;
use serde_cbor::Value as CborValue;
use url::Url;

#[derive(Clone)]
pub struct TestHelpers {
    pub console: TestConsole,
    pub channel: Box<TestChannelIntegration>,
    pub dauphin: PgDauphin,
    pub commander_inner: TestCommander,
    pub commander: PgCommander,
    pub manager: RequestManager,
    pub loader: ProgramLoader,
    pub fdr: FakeDauphinReceiver
}

impl TestHelpers {
    pub(crate) fn new() -> TestHelpers {
        let console = TestConsole::new();
        let console2 = console.clone();
        let messages = MessageSender::new(move |msg| {
            console2.message(&msg.to_string());
        });
        let booted = CountingPromise::new();
        let channel = Box::new(TestChannelIntegration::new());
        let commander_inner = TestCommander::new(&console);
        let commander = PgCommander::new(Box::new(commander_inner.clone()));
        let mut manager = RequestManager::new(channel.clone(),&commander,&messages);
        let pdq = PgDauphinQueue::new();
        let dauphin = PgDauphin::new(&pdq).expect("d");
        let fdr = FakeDauphinReceiver::new(&commander,&pdq);
        let loader = ProgramLoader::new(&commander,&manager,&dauphin);
        let stick_authority_store = StickAuthorityStore::new(&commander,&manager,&loader,&dauphin);
        let stick_store = StickStore::new(&commander,&stick_authority_store,&booted);
        manager.add_receiver(Box::new(dauphin.clone()));
        TestHelpers {
            console, channel, dauphin, commander_inner, commander,
            manager, loader, fdr
        }
    }
    
    pub(crate) fn run(&self, num: usize) {
        for _ in 0..num {
            self.commander_inner.tick();
        }
    }

    pub(crate) fn task<F>(&self, prog: F) where F: Future<Output=Result<(),DataMessage>> + 'static {
        add_task(&self.commander,PgCommanderTaskSpec {
            name: "program".to_string(),
            prio: 4,
            slot: None,
            timeout: None,
            task: Box::pin(prog)
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