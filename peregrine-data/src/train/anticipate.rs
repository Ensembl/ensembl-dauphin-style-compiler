use std::{collections::HashMap, sync::{Arc, Mutex}};
use crate::{Carriage, CarriageId, LaneStore, PeregrineCore, PeregrineCoreBase, PgCommanderTaskSpec, Scale, StickId, add_task, core::Layout, switch::trackconfiglist::TrainTrackConfigList};
use super::train::{Train, TrainId};

#[derive(Clone)]
struct AnticipatedCarriages {
    carriages: Arc<Mutex<HashMap<CarriageId,Carriage>>>
}

impl AnticipatedCarriages {
    fn new() -> AnticipatedCarriages {
        AnticipatedCarriages {
            carriages: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    fn insert(&mut self, id: &CarriageId, carriage: &Carriage) {
        self.carriages.lock().unwrap().insert(id.clone(),carriage.clone());
    }

    fn contains(&self, id: &CarriageId) -> bool {
        self.carriages.lock().unwrap().contains_key(id)
    }

    fn get(&self,id: &CarriageId) -> Option<Carriage> {
        self.carriages.lock().unwrap().get(id).cloned()
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(PartialEq,Eq)]
struct AnticipateContext {
    layout: Layout
}

impl AnticipateContext {
    fn new(layout: &Layout) -> AnticipateContext {
        AnticipateContext {
            layout: layout.clone()
        }
    }

    fn derive(&self, new_carriages: &mut AnticipatedCarriages, old_carriages: &AnticipatedCarriages, base: &PeregrineCoreBase, result_store: &LaneStore, scale: &Scale, index: u64) {
        let train_id = TrainId::new(&self.layout,&scale);
        let carriage_id = CarriageId::new(&train_id,index);
        if new_carriages.contains(&carriage_id) { return; }
        if let Some(carriage) = old_carriages.get(&carriage_id) {
            new_carriages.insert(&carriage_id,&carriage);
            return;
        }
        let train_track_config_list = TrainTrackConfigList::new(&self.layout,scale); // TODO cache
        let mut carriage = Carriage::new(&carriage_id,&train_track_config_list,None);
        new_carriages.insert(&carriage_id,&carriage);
        let base2 = base.clone();
        let result_store = result_store.clone();
        add_task(&base.commander,PgCommanderTaskSpec {
            name: format!("data program"),
            prio: 9,
            slot: None,
            timeout: None,
            stats: false,
            task: Box::pin(async move {
                carriage.load(&base2,&result_store,true).await.ok();
                Ok(())
            })
        });
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(PartialEq,Eq)]
struct AnticipatePosition {
    scale: Scale,
    index: u64,
    context: AnticipateContext,
}

impl AnticipatePosition {
    fn new(train: &Train, position: f64) -> AnticipatePosition {
        let train_id = train.id();
        let scale = train_id.scale();
        AnticipatePosition {
            scale: scale.clone(),
            index: scale.carriage(position),
            context: AnticipateContext::new(train_id.layout())
        }
    }

    fn derive(&self, new_carriages: &mut AnticipatedCarriages, old_carriages: &AnticipatedCarriages, base: &PeregrineCoreBase, result_store: &LaneStore) {
        /* out */
        let mut new_scale = self.scale.clone();
        for index in 0..5 {
            new_scale = new_scale.next_scale();
            for offset in 0..5 {
                let delta = (offset as i64)-2;
                let mut index = new_scale.convert_index(&self.scale,self.index) as i64;
                index += delta;
                if index < 0 { continue; }
                self.context.derive(new_carriages,old_carriages,base,result_store,&new_scale,index as u64);
            }
        }
        /* in */
        let mut new_scale = self.scale.clone();
        for index in 0..5 {
            new_scale = new_scale.prev_scale();
            for offset in 0..5 {
                let delta = (offset as i64)-2;
                let mut index = new_scale.convert_index(&self.scale,self.index) as i64;
                index += delta;
                if index < 0 { continue; }
                self.context.derive(new_carriages,old_carriages,base,result_store,&new_scale,index as u64);
            }
        }
    }
}

#[derive(Clone)]
pub(crate) struct Anticipate {
    base: PeregrineCoreBase,
    result_store: LaneStore,
    root: Arc<Mutex<Option<(AnticipatePosition,AnticipatedCarriages)>>>
}

impl Anticipate {
    pub(crate) fn new(base: &PeregrineCoreBase, result_store: &LaneStore) -> Anticipate {
        Anticipate {
            root: Arc::new(Mutex::new(None)),
            base: base.clone(),
            result_store: result_store.clone()
        }
    }

    pub(crate) fn anticipate(&self, train: &Train, position: f64) {
        let new_position = AnticipatePosition::new(train,position);
        let mut old_carriages = AnticipatedCarriages::new();
        if let Some((old_position,carriages)) = self.root.lock().unwrap().as_ref() {
            if &new_position == old_position { return; }
            old_carriages = carriages.clone();
        }
        let mut carriages = AnticipatedCarriages::new();
        new_position.derive(&mut carriages,&old_carriages,&self.base,&self.result_store);
        *self.root.lock().unwrap() = Some((new_position,carriages));
    }
}
