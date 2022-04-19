use std::hash::{Hash, Hasher};
use std::sync::{ Arc, Mutex };
use peregrine_toolkit::lock;
use peregrine_toolkit::sync::needed::Needed;

use crate::allotment::core::carriageoutput::CarriageOutput;
use crate::allotment::core::drawingcarriagedata::{DrawingCarriageData2};
use crate::allotment::core::trainstate::{TrainState3};
use crate::{CarriageExtent };

use lazy_static::lazy_static;
use identitynumber::identitynumber;

/* DrawingCarriages are never equal to each other because unfortunately overlapping events could mean that one is
 * being destroyed while one of the same extent is being created again.
 */
identitynumber!(IDS);

#[derive(Clone)]
pub struct DrawingCarriage {
    id: u64,
    try_lifecycle: Needed,
    extent: CarriageExtent,
    shapes: DrawingCarriageData2,
    train_state: Arc<TrainState3>,
    ready: Arc<Mutex<bool>>
}

impl PartialEq for DrawingCarriage {
    fn eq(&self, other: &Self) -> bool { self.id == other.id }
}

impl Eq for DrawingCarriage {}

impl Hash for DrawingCarriage {
    fn hash<H: Hasher>(&self, state: &mut H) { self.id.hash(state); }
}

impl DrawingCarriage {
    pub(super) fn new(extent: &CarriageExtent, try_lifecycle: &Needed, carriage_shapes: &DrawingCarriageData2, train_state: &TrainState3) -> DrawingCarriage {
        DrawingCarriage {
            extent: extent.clone(),
            try_lifecycle: try_lifecycle.clone(),
            shapes: carriage_shapes.clone(),
            id: IDS.next(),
            train_state: Arc::new(train_state.clone()),
            ready: Arc::new(Mutex::new(false))
        }
    }

    pub fn extent(&self) -> &CarriageExtent { &self.extent }
        
    pub(crate) fn is_ready(&self) -> bool { *lock!(self.ready) }
    pub fn set_ready(&self) { *lock!(self.ready) = true; self.try_lifecycle.set(); }

    pub fn solution(&self) -> &DrawingCarriageData2 { &self.shapes }
}

impl PartialEq for DrawingCarriage2 {
    fn eq(&self, other: &Self) -> bool { self.id == other.id }
}

impl Eq for DrawingCarriage2 {}

impl Hash for DrawingCarriage2 {
    fn hash<H: Hasher>(&self, state: &mut H) { self.id.hash(state); }
}

#[derive(Clone)]
pub struct DrawingCarriage2 {
    id: u64,
    try_lifecycle: Needed,
    extent: CarriageExtent,
    ready: Arc<Mutex<bool>>,
    data: DrawingCarriageData2
}

impl DrawingCarriage2 {
    pub(super) fn new(extent: &CarriageExtent, try_lifecycle: &Needed, carriage_output: &CarriageOutput, train_state: &TrainState3) -> DrawingCarriage2 {
        let data = DrawingCarriageData2::new(carriage_output,train_state);
        DrawingCarriage2 {
            id: IDS.next(),
            try_lifecycle: try_lifecycle.clone(),
            extent: extent.clone(),
            ready: Arc::new(Mutex::new(false)),
            data
        }
    }

    pub fn extent(&self) -> &CarriageExtent { &self.extent }
        
    pub(crate) fn is_ready(&self) -> bool { *lock!(self.ready) }
    pub fn set_ready(&self) { *lock!(self.ready) = true; self.try_lifecycle.set(); }

    pub fn solution(&self) -> &DrawingCarriageData2 { &self.data }
}
