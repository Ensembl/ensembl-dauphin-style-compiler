use std::hash::{Hash, Hasher};
use std::sync::{ Arc, Mutex };
use peregrine_toolkit::lock;
use peregrine_toolkit::sync::needed::Needed;
use peregrine_toolkit::sync::retainer::RetainTest;

use crate::allotment::core::carriageoutput::CarriageOutput;
use crate::allotment::core::trainstate::{TrainState3};
use crate::{Shape, LeafCommonStyle, DataMessage, TrainExtent };

use lazy_static::lazy_static;
use identitynumber::identitynumber;

use super::carriageextent::CarriageExtent;
use super::{DrawingCarriageExtent};

/* DrawingCarriages are never equal to each other because unfortunately overlapping events could mean that one is
 * being destroyed while one of the same extent is being created again.
 */
identitynumber!(IDS);

impl PartialEq for DrawingCarriage {
    fn eq(&self, other: &Self) -> bool { self.id == other.id }
}

impl Eq for DrawingCarriage {}

impl Hash for DrawingCarriage {
    fn hash<H: Hasher>(&self, state: &mut H) { self.id.hash(state); }
}

#[derive(Clone)]
pub struct DrawingCarriage {
    id: u64,
    try_lifecycle: Needed,
    extent: DrawingCarriageExtent,
    ready: Arc<Mutex<bool>>,
    shapes: Arc<Vec<Shape<LeafCommonStyle>>>,
    retain: RetainTest
}

impl DrawingCarriage {
    pub(super) fn new(extent: &CarriageExtent, try_lifecycle: &Needed, carriage_output: &CarriageOutput, train_state: &TrainState3, retain: &RetainTest) -> Result<DrawingCarriage,DataMessage> {
        let shapes = carriage_output.get(&mut *lock!(train_state.answer()))?;
        Ok(DrawingCarriage {
            id: IDS.next(),
            try_lifecycle: try_lifecycle.clone(),
            extent: DrawingCarriageExtent::new(extent.clone()),
            ready: Arc::new(Mutex::new(false)),
            shapes: Arc::new(shapes),
            retain: retain.clone()
        })
    }

    pub fn train(&self) -> &TrainExtent { &self.extent.train() }
    pub fn extent(&self) -> &DrawingCarriageExtent { &self.extent }
        
    pub(crate) fn is_ready(&self) -> bool { *lock!(self.ready) }
    pub fn set_ready(&self) { *lock!(self.ready) = true; self.try_lifecycle.set(); }

    pub fn shapes(&self) -> &Arc<Vec<Shape<LeafCommonStyle>>> { &self.shapes }
    pub fn relevancy(&self) -> RetainTest { self.retain.clone() }
}
