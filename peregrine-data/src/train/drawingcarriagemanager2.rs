use peregrine_toolkit::sync::{retainer::{Retainer, retainer, RetainTest}, needed::Needed};
use crate::{allotment::core::{trainstate::TrainState3, carriageoutput::CarriageOutput}, DrawingCarriage, DataMessage, shapeload::carriageprocess::CarriageProcess, TrainExtent, CarriageExtent};
use super::{switcher::{SwitcherManager, SwitcherExtent, SwitcherObject, Switcher}, drawingcarriagemanager::DrawingCarriageCreator, graphics::Graphics, drawingcarriageparty::DrawingCarriageParty, party::PartyState};

pub(crate) struct DrawingCarriageManager2 {
    ping_needed: Needed,
    train_extent: TrainExtent,
    active: bool,
    muted: bool,
    graphics: Graphics
}

impl DrawingCarriageManager2 {
    pub(crate) fn new(ping_needed: &Needed, extent: &TrainExtent, graphics: &Graphics) -> DrawingCarriageManager2 {
        DrawingCarriageManager2 {
            ping_needed: ping_needed.clone(),
            train_extent: extent.clone(),
            graphics: graphics.clone(),
            active: false,
            muted: false
        }
    }
}

impl SwitcherManager for DrawingCarriageManager2 {
    type Extent = TrainState3;
    type Type = DrawingCarriageParty;
    type Error = DataMessage;

    fn create(&mut self, state: &TrainState3) -> Result<DrawingCarriageParty,DataMessage> {
        let mut out = DrawingCarriageParty::new(&self.ping_needed,&self.train_extent,state,&self.graphics);
        if self.muted {
            out.set_mute();
        } else if self.active {
            out.set_active();
        }
        Ok(out)
    }

    fn busy(&self, _yn: bool) {}
}

impl SwitcherExtent for TrainState3 {
    type Extent = TrainState3;
    type Type = DrawingCarriageParty;

    fn to_milestone(&self) -> Self::Extent { self.clone() }
    fn is_milestone_for(&self, what: &Self::Extent) -> bool { self == what }
}

impl SwitcherObject for DrawingCarriageParty {
    type Extent = TrainState3;
    type Type = DrawingCarriageParty;
    type Speed = ();

    fn extent(&self) -> Self::Extent { self.state().clone() }
    fn half_ready(&self) -> bool { self.ready() }
    fn broken(&self) -> bool { false }
    fn speed(&self, _source: Option<&Self::Type>) {}
    fn live(&mut self, _speed: &()) -> bool { self.set_active(); true }
    fn dead(&mut self) { self.set_mute(); }
    fn ready(&self) -> bool { self.is_ready() }
}

pub(crate) struct DrawingCarriageSwitcher {
    switcher: Switcher<DrawingCarriageManager2,TrainState3,DrawingCarriageParty,DataMessage>,
}

impl DrawingCarriageSwitcher {
    pub(crate) fn new(ping_needed: &Needed, extent: &TrainExtent, graphics: &Graphics) -> DrawingCarriageSwitcher {
        DrawingCarriageSwitcher {
            switcher: Switcher::new(DrawingCarriageManager2::new(ping_needed,extent,graphics))
        }
    }

    pub(super) fn set_mute(&mut self) {
        self.switcher.each_mut(&|dcp| {
            dcp.set_mute();
        });
        self.switcher.manager_mut().muted = true;
    }

    pub(super) fn set_active(&mut self) {
        self.switcher.each_displayed_mut(&|dcp| {
            dcp.set_active();
        });
        self.switcher.manager_mut().active = true;
    }

    pub(super) fn set(&mut self, state: &TrainState3, carriages: &[DrawingCarriageCreator]) {
        self.switcher.set_target(state);
        self.switcher.each_mut(&|dcp| {
            dcp.set(state,carriages);
        });
    }

    pub(super) fn central_carriage(&self) -> Option<&DrawingCarriage> {
        self.switcher.displayed().and_then(|p| p.central())
    }

    pub(super) fn is_ready(&self) -> bool {
        self.switcher.displayed().is_some()
    }

    pub(super) fn ping(&mut self) {
        self.switcher.each_mut(&|dcp| {
            dcp.ping();
        });
        self.switcher.ping();
    }
}
