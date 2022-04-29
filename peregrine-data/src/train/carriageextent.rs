use crate::Region;

use super::trainextent::TrainExtent;

#[derive(Clone,Hash,PartialEq,Eq)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub(crate) struct CarriageExtent {
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

    pub(crate) fn train(&self) -> &TrainExtent { &self.train }
    pub(crate) fn index(&self) -> u64 { self.index }

    pub(crate) fn left_right(&self) -> (f64,f64) {
        let bp_in_carriage = self.train.scale().bp_in_carriage() as f64;
        let index = self.index as f64;
        (bp_in_carriage*index,bp_in_carriage*(index+1.))
    }

    pub(crate) fn region(&self) -> Region {
        Region::new(self.train.layout().stick(),self.index,self.train.scale())
    }
}

#[derive(Clone,Hash,PartialEq,Eq)]
#[cfg_attr(debug_assertions,derive(Debug))]
enum DrawingCarriageExtentData {
    Panel(CarriageExtent)
}

#[derive(Clone,Hash,PartialEq,Eq)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct DrawingCarriageExtent(DrawingCarriageExtentData);

impl DrawingCarriageExtent {
    pub(crate) fn new(x: CarriageExtent) -> DrawingCarriageExtent {
        DrawingCarriageExtent(DrawingCarriageExtentData::Panel(x))
    }

    pub fn train(&self) -> &TrainExtent {
        match &self.0 {
            DrawingCarriageExtentData::Panel(e) => e.train()
        }
    }

    pub fn left_right(&self) -> (f64,f64) {
        match &self.0 {
            DrawingCarriageExtentData::Panel(e) => e.left_right()
        }
    }
}
