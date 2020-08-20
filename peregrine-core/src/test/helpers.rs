use std::future::Future;
use super::integrations::{ TestCommander, TestChannelIntegration, TestDauphinIntegration, TestConsole };
use crate::{ PgCommander, PgCommanderTaskSpec, PgDauphin, RequestManager };
use crate::request::program::ProgramLoader;
use url::Url;

#[derive(Clone)]
pub struct TestHelpers {
    pub console: TestConsole,
    pub channel: TestChannelIntegration,
    pub dauphin: PgDauphin,
    pub commander_inner: TestCommander,
    pub commander: PgCommander,
    pub manager: RequestManager,
    pub loader: ProgramLoader
}

impl TestHelpers {
    pub(crate) fn new() -> TestHelpers {
        let console = TestConsole::new();
        let channel = TestChannelIntegration::new(&console);
        let dauphin = TestDauphinIntegration::new(&console);
        let dauphin = PgDauphin::new(Box::new(dauphin)).expect("d");
        let commander_inner = TestCommander::new(&console);
        let commander = PgCommander::new(Box::new(commander_inner.clone()));
        let manager = RequestManager::new(channel.clone(),&dauphin,&commander);
        let loader = ProgramLoader::new(&commander,&manager,&dauphin).expect("c");
        dauphin.start_runner(&commander);
        TestHelpers {
            console, channel, dauphin, commander_inner, commander,
            manager, loader
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

