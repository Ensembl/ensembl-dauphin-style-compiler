use std::future::Future;
use super::integrations::{ TestCommander, TestChannelIntegration, TestDauphinIntegration, TestConsole };
use crate::{ PgCommander, PgCommanderTaskSpec, PgDauphin, RequestManager };
use crate::index::stickstore::StickStore;
use crate::request::{ Channel, ChannelLocation };
use crate::request::program::ProgramLoader;
use serde_cbor::Value as CborValue;
use url::Url;

#[derive(Clone)]
pub struct TestHelpers {
    //pub console: TestConsole,
    pub channel: TestChannelIntegration,
    pub dauphin: PgDauphin,
    pub commander_inner: TestCommander,
    pub commander: PgCommander,
    pub manager: RequestManager,
    pub loader: ProgramLoader,
    pub messages: MessageSender
}

impl TestHelpers {
    pub(crate) fn new() -> TestHelpers {
        //let console = TestConsole::new();
        let channel = TestChannelIntegration::new(&console);
        let dauphin = TestDauphinIntegration::new(&console);
        let dauphin = PgDauphin::new(Box::new(dauphin)).expect("d");
        let commander_inner = TestCommander::new(&console);
        let commander = PgCommander::new(Box::new(commander_inner.clone()));
        let mut manager = RequestManager::new(channel.clone(),&commander);
        let stick_store = StickStore::new(&commander,&manager,&Channel::new(&ChannelLocation::HttpChannel(Url::parse("http://a.com/1").expect("e")))).expect("f"); // XXX
        let loader = ProgramLoader::new(&commander,&manager,&dauphin).expect("c");
        let messages = MessageSender::new(|msg| {});
        manager.add_receiver(Box::new(dauphin.clone()));
        manager.add_receiver(Box::new(stick_store.clone()));
        dauphin.start_runner(&commander,Box::new(console.clone()));
        TestHelpers {
            //console, 
            channel, dauphin, commander_inner, commander,
            manager, loader, messages
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