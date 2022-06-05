use peregrine_toolkit_async::{sync::{needed::Needed}};
use crate::{allotment::core::{trainstate::TrainState3, abstractcarriage::AbstractCarriage}, DrawingCarriage, DataMessage, api::api::new_train_identity, Stick, train::{core::switcher::{Switcher, SwitcherManager, SwitcherExtent, SwitcherObject}, graphics::Graphics}};

#[cfg(debug_trains)]
use peregrine_toolkit::debug_log;

use super::drawingtrain::DrawingTrain;

pub(crate) struct DrawingTrainSetActions {
    stick: Option<Stick>,
    muted: bool,
    graphics: Graphics
}

impl DrawingTrainSetActions {
    pub(crate) fn new(graphics: &Graphics) -> DrawingTrainSetActions {
        DrawingTrainSetActions {
            stick: None,
            graphics: graphics.clone(),
            muted: false
        }
    }
}

impl SwitcherManager for DrawingTrainSetActions {
    type Extent = TrainState3;
    type Type = DrawingTrain;
    type Error = DataMessage;

    fn create(&mut self, state: &TrainState3) -> Result<DrawingTrain,DataMessage> {
        let train_identity = new_train_identity();
        #[cfg(debug_trains)] debug_log!("DC party for {:x} {:?}",state.hash(),train_identity);
        let mut out = DrawingTrain::new(&train_identity,state,&self.graphics);
        if let Some(stick) = &self.stick {
            out.set_stick(stick);
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
    type Type = DrawingTrain;

    fn to_milestone(&self) -> Self::Extent { self.clone() }
    fn is_milestone_for(&self, what: &Self::Extent) -> bool { self == what }
}

impl SwitcherObject for DrawingTrain {
    type Extent = TrainState3;
    type Type = DrawingTrain;
    type Speed = ();

    fn extent(&self) -> Self::Extent { self.state().clone() }
    fn half_ready(&self) -> bool { self.ready() }
    fn broken(&self) -> bool { false }
    fn speed(&self, _source: Option<&Self::Type>) {}
    fn live(&mut self, _speed: &()) -> bool { self.set_active(); true }
    fn dead(&mut self) { self.set_mute(); }
    fn ready(&self) -> bool { self.is_ready() }
}

pub(crate) struct DrawingTrainSet {
    switcher: Switcher<DrawingTrainSetActions,TrainState3,DrawingTrain,DataMessage>,
}

impl DrawingTrainSet {
    pub(crate) fn new(graphics: &Graphics) -> DrawingTrainSet {
        DrawingTrainSet {
            switcher: Switcher::new(DrawingTrainSetActions::new(graphics))
        }
    }

    pub(crate) fn set_mute(&mut self) {
        self.switcher.each_mut(&|dcp| {
            dcp.set_mute();
        });
        self.switcher.manager_mut().muted = true;
    }

    pub(crate) fn set_active(&mut self, stick: &Stick) {
        self.switcher.manager_mut().stick = Some(stick.clone());
        self.switcher.each_mut(&|dcp| dcp.set_stick(stick));
        self.switcher.each_displayed_mut(&|dcp| {
            dcp.set_active();
        });
    }

    pub(crate) fn set(&mut self, state: &TrainState3, carriages: &[AbstractCarriage]) {
        self.switcher.set_target(state);
        self.switcher.each_mut(&|dcp| {
            dcp.set(state,carriages);
        });
    }

    pub(crate) fn central_carriage(&self) -> Option<&DrawingCarriage> {
        self.switcher.quiescent().and_then(|p| p.central())
    }

    pub(crate) fn can_be_made_active(&self) -> bool {
        self.switcher.quiescent().is_some()
    }

    pub(crate) fn ping(&mut self) {
        self.switcher.each_mut(&|dcp| {
            dcp.ping();
        });
        self.switcher.ping();
    }
}
