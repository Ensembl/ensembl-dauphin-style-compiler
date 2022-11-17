use commander::{ RunSlot, CommanderStream };
use peregrine_toolkit::error::Error;
use std::any::Any;
use std::collections::HashMap;
use peregrine_toolkit::plumbing::oneshot::OneShot;

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct PgDauphinRunTaskSpec {
    pub prio: u8, 
    pub slot: Option<RunSlot>, 
    pub timeout: Option<f64>,
    pub bundle_name: String,
    pub in_bundle_name: String,
    pub payloads: HashMap<String,Box<dyn Any>>
}

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct PgDauphinLoadTaskSpec {
    pub data: Vec<u8>,
    pub bundle_name: String
}

pub enum PgDauphinTaskSpec {
    Run(PgDauphinRunTaskSpec),
    Load(PgDauphinLoadTaskSpec),
    Quit
}

pub struct PgDauphinQueueEntry {
    pub task: PgDauphinTaskSpec,
    pub channel: CommanderStream<Result<(),Error>>
}

#[derive(Clone)]
pub struct PgDauphinQueue {
    queue: CommanderStream<PgDauphinQueueEntry>
}

impl PgDauphinQueue {
    pub fn new(shutdown: &OneShot) -> PgDauphinQueue {
        let queue = CommanderStream::new();
        let queue2 = queue.clone();
        shutdown.add(move || {
            queue2.add(PgDauphinQueueEntry {
                task: PgDauphinTaskSpec::Quit,
                channel: CommanderStream::new()
            });
        });
        PgDauphinQueue {
            queue
        }
    }

    pub async fn load(&self, task: PgDauphinLoadTaskSpec) -> Result<(),Error> {
        let waiter = CommanderStream::new();
        self.queue.add(PgDauphinQueueEntry {
            task: PgDauphinTaskSpec::Load(task),
            channel: waiter.clone()
        });
        waiter.get().await
    }

    pub async fn run(&self, task: PgDauphinRunTaskSpec) -> Result<(),Error> {
        let waiter = CommanderStream::new();
        self.queue.add(PgDauphinQueueEntry {
            task: PgDauphinTaskSpec::Run(task),
            channel: waiter.clone()
        });
        waiter.get().await
    }

    pub async fn get(&self) -> PgDauphinQueueEntry {
        self.queue.get().await
    }
}
