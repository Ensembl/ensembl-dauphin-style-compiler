use peregrine_toolkit_async::sync::needed::Needed;
use crate::{allotment::core::trainstate::TrainState3, DrawingCarriage, TrainIdentity, CarriageSpeed};
use super::{drawingcarriagemanager::{DrawingCarriageCreator, PartyDrawingCarriage}, party::{PartyActions, Party}, graphics::Graphics};
#[cfg(debug_trains)]
use lazy_static::lazy_static;
#[cfg(debug_trains)]
use identitynumber::identitynumber;

#[cfg(debug_trains)]
use peregrine_toolkit::{log, debug_log };

pub(crate) struct DrawingCarriageSetActions {
    current: Vec<PartyDrawingCarriage>,
    train_identity: TrainIdentity,
    ping_needed: Needed,
    ready: bool, /* We have initial loaded */
    active: bool,
    mute: bool,
    state: TrainState3,
    max: Option<u64>,
    graphics: Graphics,
    ready_serial: u64,
    #[cfg(debug_trains)]
    index: u64
}

#[cfg(debug_trains)]
identitynumber!(IDS);

impl DrawingCarriageSetActions {
    fn new(ping_needed: &Needed, train_identity: &TrainIdentity, state: &TrainState3, graphics: &Graphics) -> DrawingCarriageSetActions {
        DrawingCarriageSetActions {
            current: vec![],
            ping_needed: ping_needed.clone(),
            train_identity: train_identity.clone(),
            ready: false,
            active: false,
            mute: false,
            max: None,
            state: state.clone(),
            graphics: graphics.clone(),
            ready_serial: 0,
            #[cfg(debug_trains)]
            index: IDS.next()
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
        if self.active && !self.mute && self.ready {
            let carriages = self.current.iter().map(|c| c.carriage().clone()).collect::<Vec<_>>();
            self.graphics.set_carriages(&self.train_identity,&carriages);
        }
    }

    fn set_max(&mut self, max: u64) {
        self.max = Some(max);
    }

    fn transition(&mut self) {
        let max = self.max.expect("transition without max set");
        self.graphics.start_transition(&self.train_identity,max,CarriageSpeed::Quick);
    }

    fn is_ready(&self) -> bool {
        self.ready && self.max.is_some()
    }
}

impl PartyActions<DrawingCarriageCreator,PartyDrawingCarriage,PartyDrawingCarriage> for DrawingCarriageSetActions {
    fn ctor(&mut self, creator: &DrawingCarriageCreator) -> PartyDrawingCarriage {
        let carriage = PartyDrawingCarriage::new(creator,&self.train_identity,&self.state);
        #[cfg(debug_trains)] log!("DC({:x}) ctor {}",self.index,creator.extent().compact());
        if !self.mute {
            self.graphics.create_carriage(&carriage.carriage());
        }
        carriage
    }

    fn dtor_pending(&mut self, index: &DrawingCarriageCreator, item: PartyDrawingCarriage) {
        self.dtor(index,item);
    }

    fn dtor(&mut self, _index: &DrawingCarriageCreator, dc: PartyDrawingCarriage) {
        dc.destroy();
        self.ping_needed.set(); // train can maybe be updated
        #[cfg(debug_trains)] log!("DC({}) dtor {}",self.index,dc.carriage().extent().compact());
        self.graphics.drop_carriage(dc.carriage());
    }

    fn init(&mut self, _index: &DrawingCarriageCreator, carriage: &mut PartyDrawingCarriage) -> Option<PartyDrawingCarriage> {
        if !carriage.carriage().is_ready() { return None; }
        #[cfg(debug_trains)] log!("DC({:x}) init {}",self.index,carriage.carriage().extent().compact());
        self.ping_needed.set(); // train can maybe be updated
        Some(carriage.clone())
    }

    fn ready_changed(&mut self, items: &mut dyn Iterator<Item=(&DrawingCarriageCreator,&PartyDrawingCarriage)>) {
        self.ready_serial += 1;
        if !self.mute {
            self.current = items.map(|(_,y)| y.clone()).collect();
        } else {
            self.current = vec![];
        }
        self.current.sort_by_cached_key(|c| c.carriage().extent().index());
        self.try_send();
    }

    fn quiet(&mut self, _items: &mut dyn Iterator<Item=(&DrawingCarriageCreator,&PartyDrawingCarriage)>) { 
        self.ready = true;
        self.try_send();
    }
}

pub(crate) struct DrawingCarriageParty {
    slider: Party<DrawingCarriageCreator,PartyDrawingCarriage,PartyDrawingCarriage,DrawingCarriageSetActions>
}

impl DrawingCarriageParty {
    pub fn new(ping_needed: &Needed, train_identity: &TrainIdentity, state: &TrainState3, graphics: &Graphics) -> DrawingCarriageParty {
        DrawingCarriageParty {
            slider: Party::new(DrawingCarriageSetActions::new(ping_needed,train_identity,state,graphics)),
        }
    }

    pub(super) fn state(&self) -> &TrainState3 { &self.slider.inner().state }
    pub(super) fn is_ready(&self) -> bool { self.slider.inner().is_ready() }
    pub(super) fn central(&self) -> Option<&DrawingCarriage> { self.slider.inner().central() }

    pub(super) fn set(&mut self, state: &TrainState3, dcc: &[DrawingCarriageCreator]) {
        if state == self.state() {
            self.slider.set(&mut dcc.iter().cloned());
        }
    }

    pub(super) fn set_max(&mut self, max: u64) {
        self.slider.inner_mut().set_max(max);
    }

    pub(super) fn set_active(&mut self) {
        if self.slider.inner().max.is_none() {
            panic!("set_active called before set_max");
        }
        if !self.slider.inner_mut().active {
            self.slider.inner_mut().active = true;
            self.slider.inner_mut().try_send();
            self.slider.inner_mut().transition();
        }
    }

    pub(super) fn set_mute(&mut self) {
        self.slider.inner_mut().mute = true;
        #[cfg(debug_trains)] log!("DC({}) mute",self.slider.inner().index);
        self.slider.inner_mut().current = vec![];
    }

    pub(super) fn ping(&mut self) {
        self.slider.ping();
    }
}
