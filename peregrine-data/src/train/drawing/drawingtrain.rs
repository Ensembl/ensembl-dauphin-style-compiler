use peregrine_toolkit_async::sync::needed::Needed;
use crate::{allotment::core::{trainstate::TrainState3, abstractcarriage::AbstractCarriage}, DrawingCarriage, TrainIdentity, CarriageSpeed, Stick};
use super::super::{core::party::{PartyActions, Party}, graphics::Graphics};
#[cfg(debug_trains)]
use lazy_static::lazy_static;
#[cfg(debug_trains)]
use identitynumber::identitynumber;

#[cfg(debug_trains)]
use peregrine_toolkit::{log, debug_log };

pub(crate) struct DrawingTrainState {
    current: Vec<DrawingCarriage>,
    train_identity: TrainIdentity,
    ready: bool, /* We have initial loaded */
    active: bool,
    mute: bool,
    state: TrainState3,
    stick: Option<Stick>,
    graphics: Graphics,
    ready_serial: u64,
    #[cfg(debug_trains)]
    index: u64
}

#[cfg(debug_trains)]
identitynumber!(IDS);

impl DrawingTrainState {
    fn new(train_identity: &TrainIdentity, state: &TrainState3, graphics: &Graphics) -> DrawingTrainState {
        DrawingTrainState {
            current: vec![],
            train_identity: train_identity.clone(),
            ready: false,
            active: false,
            mute: false,
            stick: None,
            state: state.clone(),
            graphics: graphics.clone(),
            ready_serial: 0,
            #[cfg(debug_trains)]
            index: IDS.next()
        }
    }

    fn central(&self) -> Option<&DrawingCarriage> {
        if self.current.len() > 0 {
            Some(&self.current[self.current.len()/2])
        } else {
            None
        }
    }

    fn try_send(&self) {
        if self.active && !self.mute && self.ready {
            let carriages = self.current.iter().map(|c| c.clone()).collect::<Vec<_>>();
            self.graphics.set_carriages(&self.train_identity,&carriages);
        }
    }

    fn set_stick(&mut self, stick: &Stick) {
        self.stick = Some(stick.clone());
    }

    fn transition(&mut self) {
        let stick = self.stick.clone().expect("transition without max set");
        self.graphics.start_transition(&self.train_identity,&stick,CarriageSpeed::Quick);
    }

    fn is_ready(&self) -> bool {
        self.ready && self.stick.is_some()
    }
}

impl PartyActions<AbstractCarriage,DrawingCarriage,DrawingCarriage> for DrawingTrainState {
    fn ctor(&mut self, creator: &AbstractCarriage) -> DrawingCarriage {
        #[cfg(debug_trains)] log!("DC({:x}) ctor/1 {:?}",self.index,creator.extent().map(|x| x.compact()));
        let carriage = DrawingCarriage::new(&self.train_identity,creator,&self.state).ok().unwrap(); // XXX
        #[cfg(debug_trains)] log!("DC({:x}) ctor/2 {:?}",self.index,creator.extent().map(|x| x.compact()));
        if !self.mute {
            self.graphics.create_carriage(&carriage);
        }
        carriage
    }

    fn dtor_pending(&mut self, index: &AbstractCarriage, item: DrawingCarriage) {
        self.dtor(index,item);
    }

    fn dtor(&mut self, _index: &AbstractCarriage, mut dc: DrawingCarriage) {
        dc.destroy();
        #[cfg(debug_trains)] log!("DC({:x}) dtor {}",self.index,dc.extent().compact());
        self.graphics.drop_carriage(&dc);
    }

    fn init(&mut self, _index: &AbstractCarriage, carriage: &mut DrawingCarriage) -> Option<DrawingCarriage> {
        if !carriage.is_ready() { return None; }
        #[cfg(debug_trains)] log!("DC({:x}) init {}",self.index,carriage.extent().compact());
        Some(carriage.clone())
    }

    fn ready_changed(&mut self, items: &mut dyn Iterator<Item=(&AbstractCarriage,&DrawingCarriage)>) {
        self.ready_serial += 1;
        if !self.mute {
            self.current = items.map(|(_,y)| y.clone()).collect();
        } else {
            self.current = vec![];
        }
        self.current.sort_by_cached_key(|c| c.extent().index());
        self.try_send();
    }

    fn quiet(&mut self, _items: &mut dyn Iterator<Item=(&AbstractCarriage,&DrawingCarriage)>) { 
        self.ready = true;
        self.try_send();
    }
}

pub(crate) struct DrawingTrain {
    slider: Party<AbstractCarriage,DrawingCarriage,DrawingCarriage,DrawingTrainState>
}

impl DrawingTrain {
    pub fn new(train_identity: &TrainIdentity, state: &TrainState3, graphics: &Graphics) -> DrawingTrain {
        DrawingTrain {
            slider: Party::new(DrawingTrainState::new(train_identity,state,graphics)),
        }
    }

    pub(crate) fn state(&self) -> &TrainState3 { &self.slider.inner().state }
    pub(crate) fn is_ready(&self) -> bool { self.slider.inner().is_ready() }
    pub(crate) fn central(&self) -> Option<&DrawingCarriage> { self.slider.inner().central() }

    pub(crate) fn set(&mut self, state: &TrainState3, dcc: &[AbstractCarriage]) {
        if state == self.state() {
            self.slider.set(&mut dcc.iter().cloned());
        }
    }

    pub(crate) fn set_stick(&mut self, stick: &Stick) {
        self.slider.inner_mut().set_stick(stick);
    }

    pub(crate) fn set_active(&mut self) {
        if self.slider.inner().stick.is_none() {
            panic!("set_active called before set_stick");
        }
        if !self.slider.inner_mut().active {
            self.slider.inner_mut().active = true;
            self.slider.inner_mut().try_send();
            self.slider.inner_mut().transition();
        }
    }

    pub(crate) fn set_mute(&mut self) {
        self.slider.inner_mut().mute = true;
        #[cfg(debug_trains)] log!("DC({:x}) mute",self.slider.inner().index);
        self.slider.inner_mut().current = vec![];
    }

    pub(crate) fn ping(&mut self) {
        self.slider.ping();
    }
}
