use peregrine_toolkit_async::{sync::{needed::Needed, retainer::{Retainer, RetainTest, retainer}}};
use crate::{DrawingCarriage, allotment::core::{trainstate::TrainState3, carriageoutput::CarriageOutput}};
use super::{carriageextent::CarriageExtent};

#[derive(Clone)]
pub(super) struct DrawingCarriageCreator {
    ping_needed: Needed,
    extent: CarriageExtent,
    shapes: CarriageOutput
}

#[cfg(debug_assertions)]
impl std::fmt::Debug for DrawingCarriageCreator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DrawingCarriageCreator").field("extent", &self.extent).finish()
    }
}

impl PartialEq for DrawingCarriageCreator {
    fn eq(&self, other: &Self) -> bool {
        self.extent == other.extent
    }
}

impl Eq for DrawingCarriageCreator {}

impl std::hash::Hash for DrawingCarriageCreator {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.extent.hash(state);
    }
}

impl DrawingCarriageCreator {
    pub(super) fn new(shapes: CarriageOutput, extent: CarriageExtent, ping_needed: Needed) -> DrawingCarriageCreator {
        DrawingCarriageCreator { shapes, extent, ping_needed }
    }

    pub(super) fn create(&self, train_state: &TrainState3, retain: &RetainTest) -> DrawingCarriage {
        let carriage_spec = self.shapes.spec().ok().unwrap();
        train_state.add(self.extent.index(),&carriage_spec);
        let out = DrawingCarriage::new(&self.extent,&self.ping_needed,&self.shapes,train_state,retain).ok().unwrap();
        out
    }
}


#[derive(Clone)]
pub(super) struct PartyDrawingCarriage {
    carriage: DrawingCarriage,
    state: TrainState3,
    #[allow(unused)]
    retain: Retainer
}

impl PartyDrawingCarriage {
    pub(super) fn new(creator: &DrawingCarriageCreator, state: &TrainState3) -> PartyDrawingCarriage {
        let (retain,retain_test) = retainer();
        let carriage = creator.create(state,&retain_test);
        PartyDrawingCarriage {
            carriage, retain, state: state.clone()
        }
    }

    pub(super) fn carriage(&self) -> &DrawingCarriage { &self.carriage }

    pub(super) fn destroy(&self) {
        self.state.remove(self.carriage().extent().index());
    }
}
