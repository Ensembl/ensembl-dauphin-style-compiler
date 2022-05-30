use std::hash::{Hash, Hasher};
use std::sync::{ Arc, Mutex };
use peregrine_toolkit::lock;
use peregrine_toolkit_async::sync::needed::Needed;
use peregrine_toolkit_async::sync::retainer::{RetainTest, Retainer, retainer};
use crate::allotment::core::abstractcarriage::AbstractCarriage;
use crate::allotment::core::trainstate::{TrainState3};
use crate::shape::shape::DrawingShape;
use crate::{ DataMessage, TrainIdentity };
use lazy_static::lazy_static;
use identitynumber::identitynumber;

use super::super::model::carriageextent::CarriageExtent;

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
    train_state: TrainState3,
    shapes: Arc<Vec<DrawingShape>>,
    #[allow(unused)]
    retain: Retainer
}

impl DrawingCarriage {
    pub(crate) fn new(train_identity: &TrainIdentity, abstract_carriage: &AbstractCarriage, try_lifecycle: &Needed, train_state: &TrainState3) -> Result<DrawingCarriage,DataMessage> {
        let extent = abstract_carriage.extent().unwrap();
        let carriage_spec = abstract_carriage.spec().ok().unwrap();
        train_state.add(extent.index(),&carriage_spec);
        let shapes = abstract_carriage.make_drawing_shapes(&mut *lock!(train_state.answer()))?;
        Ok(DrawingCarriage {
            id: IDS.next(),
            try_lifecycle: try_lifecycle.clone(),
            extent: extent.clone(),
            train_state: train_state.clone(),
            ready: Arc::new(Mutex::new(false)),
            shapes: Arc::new(shapes),
            retain: retainer().0,
            train_identity: train_identity.clone(),
        })
    }

    #[cfg(debug_assertions)]
    pub fn compact(&self) -> String { 
        let index = lock!(self.train_state.answer()).serial();
        format!("({},{},{})",self.extent().train().scale().get_index(),self.extent().index(),index)
    }

    pub fn extent(&self) -> &CarriageExtent { &self.extent }
    pub fn train_identity(&self) -> &TrainIdentity { &self.train_identity }
    
    pub(crate) fn is_ready(&self) -> bool { *lock!(self.ready) }
    pub fn set_ready(&self) { *lock!(self.ready) = true; self.try_lifecycle.set(); }

    pub fn shapes(&self) -> &Arc<Vec<DrawingShape>> { &self.shapes }
    pub fn relevancy(&self) -> RetainTest { self.retain.test() }

    pub(crate) fn destroy(&mut self) {
        self.train_state.remove(self.extent().index());
    }
}
