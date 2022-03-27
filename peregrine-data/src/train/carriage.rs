use std::hash::{Hash, Hasher};
use std::sync::{ Arc, Mutex };
use peregrine_toolkit::lock;
use peregrine_toolkit::sync::needed::Needed;

use crate::allotment::core::carriageuniverse::{CarriageShapes, CarriageSolution};
use crate::allotment::core::heighttracker::HeightTracker;
use crate::allotment::core::trainstate::TrainState;
use crate::{CarriageExtent };

use lazy_static::lazy_static;
use identitynumber::identitynumber;

use super::railwayevent::RailwayEvents;

/* DrawingCarriages are never equal to each other because unfortunately overlapping events could mean that one is
 * being destroyed while one of the same extent is being created again.
 */
identitynumber!(IDS);

#[derive(Clone)]
pub struct DrawingCarriage {
    id: u64,
    try_lifecycle: Needed,
    extent: CarriageExtent,
    shapes: CarriageShapes,
    train_state: Arc<TrainState>,
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
    pub(super) fn new(extent: &CarriageExtent, railway_events: &RailwayEvents, carriage_shapes: &CarriageShapes, train_state: &TrainState) -> DrawingCarriage {
        DrawingCarriage {
            extent: extent.clone(),
            try_lifecycle: railway_events.lifecycle().clone(),
            shapes: carriage_shapes.clone(),
            id: IDS.next(),
            train_state: Arc::new(train_state.clone()),
            ready: Arc::new(Mutex::new(false))
        }
    }

    pub fn extent(&self) -> &CarriageExtent { &self.extent }
        
    pub(crate) fn is_ready(&self) -> bool { *lock!(self.ready) }
    pub fn set_ready(&self) { *lock!(self.ready) = true; self.try_lifecycle.set(); }

    pub fn solution(&self) -> CarriageSolution {
        self.shapes.get(&self.train_state)
    }

    pub fn indepentent_solution(&self) -> CarriageSolution {
        self.shapes.get(&TrainState::independent())
    }

    pub(super) fn intrinsic_height(&self) -> HeightTracker {
        self.indepentent_solution().height_tracker()
    }
}
