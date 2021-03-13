use anyhow::{ anyhow as err };
use crate::lock;
use std::sync::{ Arc, Mutex };
use crate::{ PgCommander, PgCommanderTaskSpec };
use peregrine_dauphin_queue::{ PgDauphinQueue, PgDauphinLoadTaskSpec, PgDauphinRunTaskSpec, PgDauphinTaskSpec };
use crate::util::cbor::cbor_string;
use crate::util::message::DataMessage;
use crate::run::add_task;

#[derive(Clone)]
pub struct FakeDauphinReceiver(Arc<Mutex<Vec<PgDauphinLoadTaskSpec>>>,Arc<Mutex<Vec<PgDauphinRunTaskSpec>>>);

async fn main_loop(_commander: PgCommander, fdr: FakeDauphinReceiver, pdq: PgDauphinQueue) -> Result<(),DataMessage> {
    loop {
        let e = pdq.get().await;
        let ok = match e.task {
            PgDauphinTaskSpec::Load(a) => { let ok = cbor_string(&a.data).unwrap_or("bad".to_string()) == "ok"; lock!(fdr.0).push(a); ok },
            PgDauphinTaskSpec::Run(a) => { lock!(fdr.1).push(a); true }
        };
        e.channel.add(if ok { Ok(()) } else { Err(err!("simulated error")) })
    }
}

impl FakeDauphinReceiver {
    pub fn new(commander: &PgCommander, pdq: &PgDauphinQueue) -> FakeDauphinReceiver {
        let fdr = FakeDauphinReceiver(Arc::new(Mutex::new(vec![])),Arc::new(Mutex::new(vec![])));
        add_task(&commander,PgCommanderTaskSpec {
            name: "dauphin runner".to_string(),
            prio: 2,
            slot: None,
            timeout: None,
            task: Box::pin(main_loop(commander.clone(),fdr.clone(),pdq.clone()))
        });
        fdr
    }

    pub fn take_loads(&self) -> Vec<PgDauphinLoadTaskSpec> {
        lock!(self.0).drain(..).collect()
    }

    pub fn take_runs(&self) -> Vec<PgDauphinRunTaskSpec> {
        lock!(self.1).drain(..).collect()
    }
}
