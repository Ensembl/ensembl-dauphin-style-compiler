use std::hash::{Hash, Hasher};
use std::sync::{ Arc, Mutex };
use peregrine_toolkit::lock;
use peregrine_toolkit_async::sync::needed::Needed;
use peregrine_toolkit_async::sync::retainer::RetainTest;
use crate::allotment::core::carriageoutput::CarriageOutput;
use crate::allotment::core::trainstate::{TrainState3};
use crate::{Shape, LeafStyle, DataMessage, TrainExtent, TrainIdentity };
use lazy_static::lazy_static;
use identitynumber::identitynumber;

use super::carriageextent::CarriageExtent;

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
    train_identity: TrainIdentity,
    extent: CarriageExtent,
    ready: Arc<Mutex<bool>>,
    shapes: Arc<Vec<Shape<LeafStyle>>>,
    retain: RetainTest,
    index: u64
}

impl DrawingCarriage {
    pub(super) fn new(train_identity: &TrainIdentity, extent: &CarriageExtent, try_lifecycle: &Needed, carriage_output: &CarriageOutput, train_state: &TrainState3, retain: &RetainTest) -> Result<DrawingCarriage,DataMessage> {
        let shapes = carriage_output.make(&mut *lock!(train_state.answer()))?;
        Ok(DrawingCarriage {
            id: IDS.next(),
            try_lifecycle: try_lifecycle.clone(),
            extent: extent.clone(),
            ready: Arc::new(Mutex::new(false)),
            shapes: Arc::new(shapes),
            retain: retain.clone(),
            train_identity: train_identity.clone(),
            index: lock!(train_state.answer()).serial()
        })
    }

    #[cfg(debug_assertions)]
    pub fn compact(&self) -> String { format!("({},{},{})",self.extent().train().scale().get_index(),self.extent().index(),self.index) }

    pub fn train(&self) -> &TrainExtent { &self.extent.train() }
    pub fn extent(&self) -> &CarriageExtent { &self.extent }
    pub fn train_identity(&self) -> &TrainIdentity { &self.train_identity }
    
    pub(crate) fn is_ready(&self) -> bool { *lock!(self.ready) }
    pub fn set_ready(&self) { *lock!(self.ready) = true; self.try_lifecycle.set(); }

    pub fn shapes(&self) -> &Arc<Vec<Shape<LeafStyle>>> { &self.shapes }
    pub fn relevancy(&self) -> RetainTest { self.retain.clone() }
}
