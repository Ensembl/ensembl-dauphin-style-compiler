use std::{collections::HashMap, sync::{Arc, Mutex}};
use commander::CommanderStream;

use crate::{Carriage, CarriageId, DataMessage, LaneStore, PeregrineCoreBase, PgCommanderTaskSpec, Scale, add_task, core::Layout, switch::trackconfiglist::TrainTrackConfigList, train::carriage};
use super::{carriage::CarriageLoadMode, train::{Train, TrainId}};

#[derive(Clone)]
struct AnticipatedCarriages {
    hot_carriages: Arc<Mutex<HashMap<CarriageId,Carriage>>>,
    warm_carriages: Arc<Mutex<HashMap<CarriageId,Carriage>>>,
}

impl AnticipatedCarriages {
    fn new() -> AnticipatedCarriages {
        AnticipatedCarriages {
            hot_carriages: Arc::new(Mutex::new(HashMap::new())),
            warm_carriages: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    fn insert(&mut self, id: &CarriageId, carriage: &Carriage, batch: bool) {
        let carriages = if batch { &mut self.warm_carriages } else { &mut self.hot_carriages };
        carriages.lock().unwrap().insert(id.clone(),carriage.clone());
    }

    fn contains(&self, id: &CarriageId, batch: bool) -> bool {
        if self.hot_carriages.lock().unwrap().contains_key(id) { return true; }
        if !batch { return false; }
        self.warm_carriages.lock().unwrap().contains_key(id)
    }

    fn make_carriage(&mut self, layout: &Layout, scale: &Scale, index: u64, batch: bool) -> Option<Carriage> {
        let train_id = TrainId::new(layout,&scale);
        let carriage_id = CarriageId::new(&train_id,index);
        if self.contains(&carriage_id,batch) { return None; }
        let train_track_config_list = TrainTrackConfigList::new(layout,scale); // TODO cache
        let mut carriage = Carriage::new(&carriage_id,&train_track_config_list,None);
        self.insert(&carriage_id,&carriage,batch);
        return Some(carriage);
    }

    fn carriages(&self) -> Vec<AnticipateTask> {
        let mut out = vec![];
        out.extend(self.hot_carriages.lock().unwrap().values().cloned().map(|carriage|
            AnticipateTask::Carriage(carriage,true)));
        out.push(AnticipateTask::Wait);
        out.extend(self.warm_carriages.lock().unwrap().values().cloned().map(|carriage|
            AnticipateTask::Carriage(carriage,true)));
        out.push(AnticipateTask::Wait);
        out
    }

    fn warm_carriages(&self) -> Vec<Carriage> {
        self.warm_carriages.lock().unwrap().values().cloned().collect()
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(PartialEq,Eq)]
struct AnticipatePosition {
    scale: Scale,
    index: u64,
    layout: Layout,
}

impl AnticipatePosition {
    fn new(train: &Train, position: f64) -> AnticipatePosition {
        let train_id = train.id();
        let scale = train_id.scale();
        AnticipatePosition {
            scale: scale.clone(),
            index: scale.carriage(position),
            layout: train_id.layout().clone()
        }
    }

    fn derive(&self, carriages: &mut AnticipatedCarriages, limit: i64, batch: bool) {
        /* out */
        let mut new_scale = self.scale.clone();
        for _ in 0..12.min(limit) {
            new_scale = new_scale.next_scale();
            let base_index = new_scale.convert_index(&self.scale,self.index) as i64;
            let start = (base_index - 2).max(0);
            for offset in 0..5 {
                let index = start+offset;
                if index < 0 { continue; }
                carriages.make_carriage(&self.layout,&new_scale,index as u64,batch);
            }
        }
        /* in */
        let mut new_scale = Some(self.scale.clone());
        for _index in 0..5.min(limit) {
            new_scale = new_scale.as_ref().and_then(|s| s.prev_scale());
            if let Some(new_scale) = &new_scale {
                for offset in 0..5 {
                    let delta = (offset as i64)-2;
                    let mut index = new_scale.convert_index(&self.scale,self.index) as i64;
                    index += delta;
                    if index < 0 { continue; }
                    carriages.make_carriage(&self.layout,&new_scale,index as u64,batch);
                }
            }
        }
        /* left/right */
        for offset in 2..9.min(limit) {
            let index = self.index as i64 + (offset/2) * if offset%2 == 0 { 1 } else { -1 };
            if index < 0 { continue; }
            carriages.make_carriage(&self.layout,&self.scale,index as u64,batch);
        }
    }
}

async fn anticipator(base: PeregrineCoreBase, result_store: LaneStore, stream: CommanderStream<AnticipateTask>) -> Result<(),DataMessage> {
    let mut handles = vec![];
    loop {
        let task = stream.get().await;
        match task {
            AnticipateTask::Carriage(mut carriage,batch) => {
                if batch {
                    let base2 = base.clone();
                    let result_store = result_store.clone();
                    let handle = add_task(&base.commander,PgCommanderTaskSpec {
                        name: format!("data program net"),
                        prio: 9,
                        slot: None,
                        timeout: None,
                        stats: false,
                        task: Box::pin(async move {
                            carriage.load(&base2,&result_store,CarriageLoadMode::Network).await.ok();
                            Ok(())
                        })
                    });
                    handles.push(handle);
                } else {
                    carriage.load(&base,&result_store,CarriageLoadMode::Batch).await.ok();    
                }        
            },
            AnticipateTask::Wait => {
                for handle in handles.drain(..) {
                    handle.finish_future().await;
                }
            }
        }
    }
}

fn run_anticipator(base: &PeregrineCoreBase, result_store: &LaneStore, stream: &CommanderStream<AnticipateTask>) {
    let stream = stream.clone();
    let base2 = base.clone();
    let result_store = result_store.clone();
    add_task(&base.commander,PgCommanderTaskSpec {
        name: format!("anticipator"),
        prio: 9,
        slot: None,
        timeout: None,
        stats: false,
        task: Box::pin(async move {
            anticipator(base2,result_store,stream).await
        })
    });
}

enum AnticipateTask {
    Carriage(Carriage,bool),
    Wait
}

#[derive(Clone)]
pub(crate) struct Anticipate {
    position: Arc<Mutex<Option<AnticipatePosition>>>,
    stream: CommanderStream<AnticipateTask>
}

impl Anticipate {
    pub(crate) fn new(base: &PeregrineCoreBase, result_store: &LaneStore) -> Anticipate {
        let stream = CommanderStream::new();
        run_anticipator(&base,&result_store,&stream);
        Anticipate {
            position: Arc::new(Mutex::new(None)),
            stream
        }
    }

    fn lightweight(&self) -> bool {
        cfg!(debug_assertions)
    }

    pub(crate) fn anticipate(&self, train: &Train, position: f64) {
        let new_position = AnticipatePosition::new(train,position);
        if let Some(old_position) = self.position.lock().unwrap().as_ref() {
            if &new_position == old_position { return; }
        }
        let mut carriages = AnticipatedCarriages::new();
        new_position.derive(&mut carriages,4,true);
        new_position.derive(&mut carriages,4,false);
        if !self.lightweight() {
            new_position.derive(&mut carriages,100,true);
        }
        *self.position.lock().unwrap() = Some(new_position);
        for task in carriages.carriages() {
            self.stream.add(task);
        }
    }
}
