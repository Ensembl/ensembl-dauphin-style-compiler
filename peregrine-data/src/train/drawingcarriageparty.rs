use peregrine_toolkit::{sync::needed::Needed, log};
use crate::{allotment::core::trainstate::TrainState3, TrainExtent, DrawingCarriage, Track};
use super::{drawingcarriagemanager::{DrawingCarriageCreator, PartyDrawingCarriage}, party::{PartyActions, Party}, graphics::Graphics};

#[cfg(debug_trains)]
use peregrine_toolkit::debug_log;

pub(crate) struct DrawingCarriageSetActions {
    current: Vec<PartyDrawingCarriage>,
    ping_needed: Needed,
    ready: bool, /* We have initial loaded */
    active: bool,
    mute: bool,
    state: TrainState3,
    extent: TrainExtent,
    graphics: Graphics,
    ready_serial: u64
}

impl DrawingCarriageSetActions {
    fn new(ping_needed: &Needed, extent: &TrainExtent, state: &TrainState3, graphics: &Graphics) -> DrawingCarriageSetActions {
        DrawingCarriageSetActions {
            current: vec![],
            ping_needed: ping_needed.clone(),
            ready: false,
            active: false,
            mute: false,
            state: state.clone(),
            extent: extent.clone(),
            graphics: graphics.clone(),
            ready_serial: 0
        }
    }

    fn central(&self) -> Option<&DrawingCarriage> {
        if self.current.len() > 0 {
            Some(&self.current[self.current.len()/2].carriage())
        } else {
            None
        }
    }

    fn try_send(&self) {
        if self.active && !self.mute {
            let carriages = self.current.iter().map(|c| c.carriage().clone()).collect::<Vec<_>>();
            self.graphics.set_carriages(&self.extent,&carriages);
        }
    }
}

impl PartyActions<DrawingCarriageCreator,PartyDrawingCarriage,PartyDrawingCarriage> for DrawingCarriageSetActions {
    fn ctor(&mut self, creator: &DrawingCarriageCreator) -> PartyDrawingCarriage {
        let carriage = PartyDrawingCarriage::new(creator,&self.state);
        if !self.mute {
            self.graphics.create_carriage(&carriage.carriage());
        }
        carriage
    }

    fn dtor(&mut self, _index: &DrawingCarriageCreator, dc: PartyDrawingCarriage) {
        self.ping_needed.set(); // train can maybe be updated
        self.graphics.drop_carriage(dc.carriage());
    }

    fn init(&mut self, _index: &DrawingCarriageCreator, item: &mut PartyDrawingCarriage) -> Option<PartyDrawingCarriage> {
        if !item.carriage().is_ready() { return None; }
        self.ping_needed.set(); // train can maybe be updated
        Some(item.clone())
    }

    fn ready_changed(&mut self, items: &mut dyn Iterator<Item=(&DrawingCarriageCreator,&PartyDrawingCarriage)>) {
        self.ready_serial += 1;
        self.current = items.map(|(_,y)| y.clone()).collect();
        self.current.sort_by_cached_key(|c| c.carriage().extent().index());
        self.try_send();
    }

    fn quiet(&mut self, _items: &mut dyn Iterator<Item=(&DrawingCarriageCreator,&PartyDrawingCarriage)>) { 
        self.ready = true;
    }
}

pub(crate) struct DrawingCarriageParty {
    slider: Party<DrawingCarriageCreator,PartyDrawingCarriage,PartyDrawingCarriage,DrawingCarriageSetActions>
}

impl DrawingCarriageParty {
    pub fn new(ping_needed: &Needed, extent: &TrainExtent, state: &TrainState3, graphics: &Graphics) -> DrawingCarriageParty {
        DrawingCarriageParty {
            slider: Party::new(DrawingCarriageSetActions::new(ping_needed,extent,state,graphics))
        }
    }

    pub(super) fn state(&self) -> &TrainState3 { &self.slider.inner().state }
    pub(super) fn is_ready(&self) -> bool { self.slider.inner().ready }
    pub(super) fn central(&self) -> Option<&DrawingCarriage> { self.slider.inner().central() }

    pub(super) fn set(&mut self, state: &TrainState3, dcc: &[DrawingCarriageCreator]) {
        if state == self.state() {
            self.slider.set(&mut dcc.iter().cloned());
        }
    }

    pub(super) fn set_active(&mut self) {
        self.slider.inner_mut().active = true;
        self.slider.inner_mut().try_send();
    }

    pub(super) fn set_mute(&mut self) {
        self.slider.inner_mut().mute = true;
    }

    pub(super) fn ping(&mut self) {
        self.slider.ping();
    }
}
