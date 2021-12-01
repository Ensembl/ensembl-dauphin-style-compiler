use crate::Region;

use super::trainextent::TrainExtent;

#[derive(Clone,Hash,PartialEq,Eq)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct CarriageExtent {
    train: TrainExtent,
    index: u64
}

impl CarriageExtent {
    pub fn new(train_id: &TrainExtent, index: u64) -> CarriageExtent {
        CarriageExtent {
            train: train_id.clone(),
            index
        }
    }

    pub fn train(&self) -> &TrainExtent { &self.train }
    pub fn index(&self) -> u64 { self.index }

    pub fn left_right(&self) -> (f64,f64) {
        let bp_in_carriage = self.train.scale().bp_in_carriage() as f64;
        let index = self.index as f64;
        (bp_in_carriage*index,bp_in_carriage*(index+1.))
    }

    pub fn region(&self) -> Region {
        Region::new(self.train.layout().stick(),self.index,self.train.scale())
    }
}