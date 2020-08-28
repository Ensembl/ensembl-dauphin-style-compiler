use anyhow;
use blackbox::blackbox_log;
use commander::{ RunSlot, CommanderStream };
use serde_cbor::Value as CborValue;
use std::any::Any;
use std::collections::HashMap;

#[derive(Debug)]
pub struct PgDauphinRunTaskSpec {
    pub prio: i8, 
    pub slot: Option<RunSlot>, 
    pub timeout: Option<f64>,
    pub bundle_name: String,
    pub in_bundle_name: String,
    pub payloads: HashMap<String,Box<dyn Any>>
}

#[derive(Debug)]
pub struct PgDauphinLoadTaskSpec {
    pub data: CborValue,
    pub bundle_name: String
}

pub enum PgDauphinTaskSpec {
    Run(PgDauphinRunTaskSpec),
    Load(PgDauphinLoadTaskSpec)
}

pub struct PgDauphinQueueEntry {
    pub task: PgDauphinTaskSpec,
    pub channel: CommanderStream<anyhow::Result<()>>
}

#[derive(Clone)]
pub struct PgDauphinQueue {
    queue: CommanderStream<PgDauphinQueueEntry>
}

impl PgDauphinQueue {
    pub fn new() -> PgDauphinQueue {
        PgDauphinQueue {
            queue: CommanderStream::new()
        }
    }

    pub async fn load(&self, task: PgDauphinLoadTaskSpec) -> anyhow::Result<()> {
        blackbox_log!("dauphin","queueing load {}",task.bundle_name);
        let waiter = CommanderStream::new();
        self.queue.add(PgDauphinQueueEntry {
            task: PgDauphinTaskSpec::Load(task),
            channel: waiter.clone()
        });
        let out = waiter.get().await;
        blackbox_log!("dauphin","finished loading {} success={:?}",name,out.is_ok());
        out
    }

    pub async fn run(&self, task: PgDauphinRunTaskSpec) -> anyhow::Result<()> {
        blackbox_log!("dauphin","queueing run {} {}",task.bundle_name,task.in_bundle_name);
        let waiter = CommanderStream::new();
        self.queue.add(PgDauphinQueueEntry {
            task: PgDauphinTaskSpec::Run(task),
            channel: waiter.clone()
        });
        let out = waiter.get().await;
        blackbox_log!("dauphin","finished {} {} success={:?}",name.0,name.1,out.is_ok());
        out
    }

    pub async fn get(&self) -> PgDauphinQueueEntry {
        self.queue.get().await
    }
}
