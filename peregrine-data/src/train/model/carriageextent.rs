use crate::{Region, Scale};

use super::trainextent::TrainExtent;

#[derive(Clone,Hash,PartialEq,Eq)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct CarriageExtent {
    train: TrainExtent,
    index: u64
}

impl CarriageExtent {
    pub(crate) fn new(train_id: &TrainExtent, index: u64) -> CarriageExtent {
        CarriageExtent {
            train: train_id.clone(),
            index
        }
    }

    #[cfg(any(debug_assertions,debug_trains))]
    pub fn compact(&self) -> String { format!("({},{})",self.train().scale().get_index(),self.index()) }

    pub fn scale(&self) -> &Scale { self.train.scale() }
    pub(crate) fn train(&self) -> &TrainExtent { &self.train }
    pub(crate) fn index(&self) -> u64 { self.index }

    pub fn left_right(&self) -> (f64,f64) {
        let bp_in_carriage = self.train.scale().bp_in_carriage() as f64;
        let index = self.index as f64;
        (bp_in_carriage*index,bp_in_carriage*(index+1.))
    }

    pub(crate) fn region(&self) -> Region {
        Region::new(self.train.layout().stick(),self.index,self.train.scale())
    }
}
