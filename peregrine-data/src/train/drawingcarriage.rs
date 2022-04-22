use std::hash::{Hash, Hasher};
use std::sync::{ Arc, Mutex };
use peregrine_toolkit::lock;
use peregrine_toolkit::sync::needed::Needed;
use peregrine_toolkit::sync::retainer::RetainTest;

use crate::allotment::core::carriageoutput::CarriageOutput;
use crate::allotment::core::trainstate::{TrainState3};
use crate::{CarriageExtent, Shape, LeafCommonStyle, DataMessage };

use lazy_static::lazy_static;
use identitynumber::identitynumber;

/* DrawingCarriages are never equal to each other because unfortunately overlapping events could mean that one is
 * being destroyed while one of the same extent is being created again.
 */
identitynumber!(IDS);

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
    shapes: Arc<Vec<Shape<LeafCommonStyle>>>,
    retain: RetainTest
}

impl DrawingCarriage2 {
    pub(super) fn new(extent: &CarriageExtent, try_lifecycle: &Needed, carriage_output: &CarriageOutput, train_state: &TrainState3, retain: &RetainTest) -> Result<DrawingCarriage2,DataMessage> {
        let shapes = carriage_output.get(&mut *lock!(train_state.answer()))?;
        Ok(DrawingCarriage2 {
            id: IDS.next(),
            try_lifecycle: try_lifecycle.clone(),
            extent: extent.clone(),
            ready: Arc::new(Mutex::new(false)),
            shapes: Arc::new(shapes),
            retain: retain.clone()
        })
    }

    pub fn extent(&self) -> &CarriageExtent { &self.extent }
        
    pub(crate) fn is_ready(&self) -> bool { *lock!(self.ready) }
    pub fn set_ready(&self) { *lock!(self.ready) = true; self.try_lifecycle.set(); }

    pub fn shapes(&self) -> &Arc<Vec<Shape<LeafCommonStyle>>> { &self.shapes }
    pub fn relevancy(&self) -> RetainTest { self.retain.clone() }
}
