use std::sync::{ Arc, Mutex };
use crate::PgCommanderTaskSpec;
use crate::core::{ PeregrineData, Scale };
use super::train::{ Train, TrainId, RailwayId };

/* current: train currently being displayed, if any. During transition, the outgoing train.
 * future: incoming train during transition.
 * wanted: train to be displayed when complete and when transitions are done.
 */

pub struct TrainSetData {
    current: Option<Train>,
    future: Option<Train>,
    wanted: Option<Train>
}

impl TrainSetData {
    pub fn new() -> TrainSetData {
        TrainSetData {
            current: None,
            future: None,
            wanted: None
        }
    }

    fn promote_future(&mut self) {
        if self.current.is_none() {
            self.current = self.future.take();
        }
    }

    fn quick(&self) -> bool {
        if let Some(current) = &self.current {
            if let Some(future) = &self.future {
                if current.id().railway() != future.id().railway() {
                    return false;
                }
            }
        }
        true
    }

    fn promote_wanted(&mut self, data: &mut PeregrineData) {
        if self.future.is_none() && self.wanted.as_ref().map(|x| x.ready()).unwrap_or(false) {
            self.future = self.wanted.take();
            let carriages = self.future.as_ref().unwrap().carriages();
            data.integration.lock().unwrap().set_carriages(&carriages,self.quick());
        }
    }

    fn promote(&mut self, data: &mut PeregrineData) {
        self.promote_future();
        self.promote_wanted(data);
        self.promote_future();
    }

    fn new_wanted(&mut self, data: &mut PeregrineData, train_id: &TrainId, position: f64) {
        self.wanted = Some(Train::new(data,train_id,position));
    }

    fn set(&mut self, data: &mut PeregrineData, railway_id: &RailwayId, position: f64, scale: f64) -> Option<Train> {
        let quiescent = if let Some(wanted) = &mut self.wanted {
            Some(wanted)
        } else if let Some(future) = &mut self.future {
            Some(future)
        } else if let Some(current) = &mut self.current {
            Some(current)
        } else {
            None
        };
        let train_id = TrainId::new(railway_id,&Scale::new_for_numeric(scale));
        let mut changed = None;
        if let Some(quiescent) = quiescent {
            if quiescent.id() == train_id {
                if quiescent.set_position(data,position) {
                    changed = Some(quiescent.clone());
                }
            } else {
                self.new_wanted(data,&train_id,position);
                changed = Some(self.wanted.as_ref().unwrap().clone());
            }
        } else {
            self.new_wanted(data,&train_id,position);
            changed = Some(self.wanted.as_ref().unwrap().clone());
        }
        self.promote(data);
        changed
    }

    pub fn transition_complete(&mut self, data: &mut PeregrineData) {
        self.current = None;
        self.promote(data);
    }
}

#[derive(Clone)]
pub struct TrainSet(Arc<Mutex<TrainSetData>>);

impl TrainSet {
    pub fn new() -> TrainSet {
        TrainSet(Arc::new(Mutex::new(TrainSetData::new())))
    }

    fn run_loading(&self, data: &mut PeregrineData, train: Train) {
        let tdata = self.0.clone();
        let pdata = data.clone();
        let mut pdata2 = pdata.clone();
        let train2 = train.clone();
        data.commander.add_task(PgCommanderTaskSpec {
            name: format!("train loader: {}",train.id()),
            prio: 1,
            slot: None,
            timeout: None,
            task: Box::pin(async move {
                train2.load(&mut pdata2).await;
                tdata.lock().unwrap().promote(&mut pdata2);
                Ok(())
            })
        });
    }

    pub fn set(&self, data: &mut PeregrineData, railway_id: &RailwayId, position: f64, scale: f64) {
        let changed = self.0.lock().unwrap().set(data,railway_id,position,scale);
        if let Some(train) = changed {
            self.run_loading(data,train);
        }
    }

    pub fn transition_complete(&self, data: &mut PeregrineData) {
        self.0.lock().unwrap().transition_complete(data);
    }
}
