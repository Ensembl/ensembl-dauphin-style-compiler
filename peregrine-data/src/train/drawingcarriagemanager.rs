use peregrine_toolkit::{sync::{needed::Needed, retainer::{Retainer, RetainTest, retainer}}, debug_log, log};
use crate::{TrainExtent, DrawingCarriage, allotment::core::{trainstate::TrainState3, carriageoutput::CarriageOutput}};
use super::{graphics::Graphics, carriageextent::CarriageExtent, party::PartyActions};

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
        let out = DrawingCarriage::new(&self.extent,&self.ping_needed,&self.shapes,train_state,retain);
        out.ok().unwrap()
    }

    pub(super) fn extent(&self) -> &CarriageExtent { &self.extent }
}

pub(super) struct DrawingCarriageManager {
    active: bool,
    mute: bool,
    ping_needed: Needed,
    train_extent: TrainExtent,
    carriages: Vec<DrawingCarriage>,
    graphics: Graphics
}

impl DrawingCarriageManager {
    pub(super) fn new(ping_needed: &Needed, train_extent: &TrainExtent, graphics: &Graphics) -> DrawingCarriageManager {
        DrawingCarriageManager {
            active: false,
            mute: false,
            ping_needed: ping_needed.clone(),
            train_extent: train_extent.clone(),
            carriages: vec![],
            graphics: graphics.clone()
        }
    }

    fn send_carriages(&self) {
        if self.active && !self.mute && self.carriages.len() > 0 {
            self.graphics.set_carriages(&self.train_extent,&self.carriages);
        }
    }

    pub(super) fn mute(&mut self, yn: bool) {
        self.mute = yn;
    }

    pub(super) fn is_active(&self) -> bool { self.active }

    pub(super) fn set_active(&mut self) {
        self.active = true;
        self.send_carriages();
    }
}

#[derive(Clone)]
pub(super) struct PartyDrawingCarriage {
    carriage: DrawingCarriage,
    #[allow(unused)]
    retain: Retainer
}

impl PartyDrawingCarriage {
    pub(super) fn new(creator: &DrawingCarriageCreator, state: &TrainState3) -> PartyDrawingCarriage {
        let (retain,retain_test) = retainer();
        let carriage = creator.create(state,&retain_test);
        PartyDrawingCarriage {
            carriage, retain
        }
    }

    pub(super) fn carriage(&self) -> &DrawingCarriage { &self.carriage }
}

impl PartyActions<(DrawingCarriageCreator,TrainState3),PartyDrawingCarriage,PartyDrawingCarriage> for DrawingCarriageManager {
    fn ctor(&mut self, (creator,state): &(DrawingCarriageCreator,TrainState3)) -> PartyDrawingCarriage {
        let carriage = PartyDrawingCarriage::new(creator,state);
        if !self.mute {
            self.graphics.create_carriage(&carriage.carriage);
        }
        carriage
    }

    fn init(&mut self, _: &(DrawingCarriageCreator,TrainState3), item: &mut PartyDrawingCarriage) -> Option<PartyDrawingCarriage> {
        if !item.carriage.is_ready() { return None; }
        self.ping_needed.set(); // train can maybe be updated
        Some(item.clone())
    }

    fn quiet(&mut self, items: &mut dyn Iterator<Item=(&(DrawingCarriageCreator, TrainState3), &PartyDrawingCarriage)>) {
        self.carriages = items.map(|x| &x.1.carriage).cloned().collect::<Vec<_>>();
        #[cfg(debug_trains)] debug_log!("set dcs {:?}",self.carriages.iter().map(|x| x.extent()).collect::<Vec<_>>());
        self.send_carriages();
    }

    fn dtor(&mut self, (dcc,state): &(DrawingCarriageCreator,TrainState3), dc: PartyDrawingCarriage) {
        #[cfg(debug_trains)] debug_log!("drop dc {:?} {:?}",dcc.extent,state);
        self.graphics.drop_carriage(&dc.carriage);
    }
}
