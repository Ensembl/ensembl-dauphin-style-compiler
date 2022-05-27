use peregrine_toolkit::{log, warn};
use peregrine_toolkit_async::{sync::{needed::Needed}};
use crate::{allotment::core::{trainstate::TrainState3}, DrawingCarriage, DataMessage, TrainExtent, TrainIdentity, api::api::new_train_identity, CarriageSpeed};
use super::{switcher::{SwitcherManager, SwitcherExtent, SwitcherObject, Switcher}, drawingcarriagemanager::DrawingCarriageCreator, graphics::Graphics, drawingcarriageparty::DrawingCarriageParty};

#[cfg(debug_trains)]
use peregrine_toolkit::debug_log;

pub(crate) struct DrawingCarriageManager2 {
    ping_needed: Needed,
    max: Option<u64>,
    muted: bool,
    graphics: Graphics
}

impl DrawingCarriageManager2 {
    pub(crate) fn new(ping_needed: &Needed, graphics: &Graphics) -> DrawingCarriageManager2 {
        DrawingCarriageManager2 {
            ping_needed: ping_needed.clone(),
            max: None,
            graphics: graphics.clone(),
            muted: false
        }
    }
}

impl SwitcherManager for DrawingCarriageManager2 {
    type Extent = TrainState3;
    type Type = DrawingCarriageParty;
    type Error = DataMessage;

    fn create(&mut self, state: &TrainState3) -> Result<DrawingCarriageParty,DataMessage> {
        let train_identity = new_train_identity();
        #[cfg(debug_trains)] debug_log!("DC party for {:x} {:?}",state.hash(),self.train_extent.scale());
        let mut out = DrawingCarriageParty::new(&self.ping_needed,&train_identity,state,&self.graphics);
        if let Some(max) = self.max {
            out.set_max(max);
        }
        if self.muted {
            out.set_mute();
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
    pub(crate) fn new(ping_needed: &Needed, graphics: &Graphics) -> DrawingCarriageSwitcher {
        DrawingCarriageSwitcher {
            switcher: Switcher::new(DrawingCarriageManager2::new(ping_needed,graphics))
        }
    }

    pub(super) fn set_mute(&mut self) {
        self.switcher.each_mut(&|dcp| {
            dcp.set_mute();
        });
        self.switcher.manager_mut().muted = true;
    }

    pub(super) fn set_active(&mut self, max: u64) {
        self.switcher.manager_mut().max = Some(max);
        self.switcher.each_mut(&|dcp| dcp.set_max(max));
        self.switcher.each_displayed_mut(&|dcp| {
            dcp.set_active();
        });
    }

    pub(super) fn set(&mut self, state: &TrainState3, carriages: &[DrawingCarriageCreator]) {
        self.switcher.set_target(state);
        self.switcher.each_mut(&|dcp| {
            dcp.set(state,carriages);
        });
    }

    pub(super) fn central_carriage(&self) -> Option<&DrawingCarriage> {
        self.switcher.quiescent().and_then(|p| p.central())
    }

    pub(super) fn can_be_made_active(&self) -> bool {
        self.switcher.quiescent().is_some()
    }

    pub(super) fn ping(&mut self) {
        self.switcher.each_mut(&|dcp| {
            dcp.ping();
        });
        self.switcher.ping();
    }
}
