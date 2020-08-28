use std::future::Future;
use std::rc::Rc;
use super::integrations::{ TestCommander, TestChannelIntegration, TestConsole, FakeDauphinReceiver };
use crate::{ PgCommander, PgCommanderTaskSpec, PgDauphin, RequestManager };
use crate::{ Channel, ChannelLocation, ProgramLoader, StickStore, StickAuthorityStore, CountingPromise };
use peregrine_dauphin_queue::PgDauphinQueue;
use serde_cbor::Value as CborValue;
use url::Url;

#[derive(Clone)]
pub struct TestHelpers {
    pub console: TestConsole,
    pub channel: TestChannelIntegration,
    pub dauphin: PgDauphin,
    pub commander_inner: TestCommander,
    pub commander: PgCommander,
    pub manager: RequestManager,
    pub loader: ProgramLoader,
    pub fdr: FakeDauphinReceiver
}

impl TestHelpers {
    pub(crate) fn new() -> TestHelpers {
        let booted = CountingPromise::new();
        let console = TestConsole::new();
        let channel = TestChannelIntegration::new(&console);
        let commander_inner = TestCommander::new(&console);
        let commander = PgCommander::new(Box::new(commander_inner.clone()));
        let mut manager = RequestManager::new(channel.clone(),&commander);
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

    pub(crate) fn task<F>(&self, prog: F) where F: Future<Output=anyhow::Result<()>> + 'static {
        self.commander.add_task(PgCommanderTaskSpec {
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