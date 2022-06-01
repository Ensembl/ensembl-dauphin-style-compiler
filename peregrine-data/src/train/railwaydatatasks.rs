use std::{sync::{Arc, Mutex}, mem};
use peregrine_toolkit_async::sync::needed::Needed;
use peregrine_toolkit::lock;
use crate::{shapeload::{carriagebuilder::CarriageBuilder, loadshapes::LoadMode}, add_task, PgCommanderTaskSpec, async_complete_task, PeregrineCoreBase, ShapeStore, DataMessage, StickStore };
use super::{model::trainextent::TrainExtent, main::{train::StickData, railway::Railway} };

#[cfg(debug_trains)]
use peregrine_toolkit::log;

#[derive(Clone)]
struct RailwayPinger(Arc<Mutex<Option<Railway>>>);

impl RailwayPinger {
    fn new() -> RailwayPinger { RailwayPinger(Arc::new(Mutex::new(None))) }

    fn set_railway(&mut self, railway: &Railway) {
       *lock!(self.0) = Some(railway.clone());
    }

    fn ping(&self) {
        let railway = lock!(self.0).clone();
        railway.unwrap().ping();
    }
}

enum Task {
    Carriage(CarriageBuilder),
    Stick(TrainExtent,Arc<Mutex<StickData>>)
}

async fn load_one_carriage(base: &mut PeregrineCoreBase, shape_store: &ShapeStore, mut carriage: CarriageBuilder) -> Result<(),DataMessage> {
    let r = carriage.load(base,&shape_store,LoadMode::RealTime).await;
    r
}

async fn load_one_stick(base: &mut PeregrineCoreBase, stick_store: &StickStore, train_extent: &TrainExtent, stick_data: &Arc<Mutex<StickData>>) -> Result<(),DataMessage> {
    let output = stick_store.get(&train_extent.layout().stick()).await;
    let data = match output {
        Ok(value) => StickData::Ready(value),
        Err(e) => {
            base.messages.send(e.clone());
            StickData::Unavailable
        }
    };
    *lock!(stick_data) = data;
    Ok(())
}


#[derive(Clone)]
pub(crate) struct RailwayDataTasks {
    tasks: Arc<Mutex<Vec<Task>>>,
    base: PeregrineCoreBase,
    shape_store: ShapeStore,
    stick_store: StickStore,
    pinger: RailwayPinger,
    ping_needed: Needed
}

impl RailwayDataTasks {
    pub fn new(base: &PeregrineCoreBase, shape_store: &ShapeStore, stick_store: &StickStore, ping_needed: &Needed) -> RailwayDataTasks {
        RailwayDataTasks {
            base: base.clone(),
            ping_needed: ping_needed.clone(),
            shape_store: shape_store.clone(),
            stick_store: stick_store.clone(),
            pinger: RailwayPinger::new(),
            tasks: Arc::new(Mutex::new(vec![]))
        }
    }

    pub(crate) fn set_railway(&mut self, railway: &Railway) {
        self.pinger.set_railway(railway);
    }

    pub(crate) fn add_carriage(&self, carriage: &CarriageBuilder) {
        lock!(self.tasks).push(Task::Carriage(carriage.clone()));
    }

    pub(crate) fn add_stick(&self, train_extent: &TrainExtent, output: &Arc<Mutex<StickData>>) {
        lock!(self.tasks).push(Task::Stick(train_extent.clone(),output.clone()));
    }

    fn async_load(&self, task: Task) {
        let self2 = self.clone();
        let mut base2 = self.base.clone();
        let ping_needed = self.ping_needed.clone();
        let shape_store = self.shape_store.clone();
        let stick_store = self.stick_store.clone();
        let pinger = self.pinger.clone();
        let handle = add_task(&self.base.commander,PgCommanderTaskSpec {
            name: format!("carriage loader"),
            prio: 1,
            slot: None,
            timeout: None,
            task: Box::pin(async move {
                let result = match task {
                    Task::Carriage(carriage) => {
                        load_one_carriage(&mut base2,&shape_store,carriage).await
                    },
                    Task::Stick(extent,stick) => {
                        load_one_stick(&mut base2,&stick_store,&extent,&stick).await
                    }
                };
                pinger.ping();
                ping_needed.set();
                self2.load();
                if let Err(e) = result {
                    base2.messages.send(e.clone());
                }
                Ok(())
            }),
            stats: false
        });
        async_complete_task(&self.base.commander, &self.base.messages,handle,|e| (e,false));
    }

    pub(crate) fn load(&self) {
        let mut tasks = mem::replace(&mut *lock!(self.tasks), vec![]);
        for task in tasks.drain(..) {
            self.async_load(task);
        }
    }
}
