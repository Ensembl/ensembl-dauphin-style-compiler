use crate::PgCommander;
use crate::index::StickStore;
use crate::panel::PanelStore;
use crate::train::{ Carriage, TrainSet };
use crate::PgCommanderTaskSpec;
use std::sync::{ Arc, Mutex };
use commander::CommanderStream;

#[derive(Clone)]
pub struct PeregrineData {
    pub commander: PgCommander,
    pub panel_store: PanelStore,
    pub integration: Arc<Mutex<Box<dyn PeregrineIntegration>>>,
    pub train_set: TrainSet,
    pub stick_store: StickStore
}

pub enum ApiMessage {
    TransitionComplete
}

pub struct PeregrineApi {
    queue: PeregrineApiQueue
}

impl PeregrineApi {
    pub fn transition_complete(&self) {
        self.queue.push(ApiMessage::TransitionComplete);
    }
}

#[derive(Clone)]
pub struct PeregrineApiQueue {
    queue: CommanderStream<ApiMessage>,
    data: PeregrineData
}

impl PeregrineApiQueue {
    pub fn new(data:&PeregrineData) -> PeregrineApiQueue {
        PeregrineApiQueue {
            queue: CommanderStream::new(),
            data: data.clone()
        }
    }

    fn run_message(&mut self, message: ApiMessage) {
        match message {
            ApiMessage::TransitionComplete => {
                let train_set = self.data.train_set.clone();
                train_set.transition_complete(&mut self.data);
            }
        }
    }

    pub fn run(&self) {
        let mut self2 = self.clone();
        self.data.commander.add_task(PgCommanderTaskSpec {
            name: format!("api message runner"),
            prio: 5,
            slot: None,
            timeout: None,
            task: Box::pin(async move {
                loop {
                    let message = self2.queue.get().await;
                    self2.run_message(message);
                }
            })
        });
    }

    pub fn push(&self, message: ApiMessage) {
        self.queue.add(message);
    }
}

pub trait PeregrineIntegration {
    fn set_api(&self, api: PeregrineApi);
    fn report_error(&self, error: &str);
    fn set_carriages(&self, carriages: &[Carriage], quick: bool);
}
