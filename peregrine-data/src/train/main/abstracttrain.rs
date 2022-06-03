use std::{sync::{Arc, Mutex}, cmp::max};
use peregrine_toolkit::{puzzle::AnswerAllocator};
use peregrine_toolkit_async::sync::needed::Needed;
use crate::{shapeload::carriagebuilder::CarriageBuilder, allotment::core::{trainstate::{TrainStateSpec, TrainState3}, abstractcarriage::AbstractCarriage}, switch::trackconfiglist::TrainTrackConfigList, api::MessageSender, CarriageExtent, train::{model::trainextent::TrainExtent, graphics::Graphics, core::party::{PartyActions, Party, PartyState}}, PeregrineApiQueue};

#[cfg(debug_trains)]
use peregrine_toolkit::{ log, debug_log };

#[cfg(no_flank)]
const FLANK : u64 = 0;

#[cfg(not(no_flank))]
const FLANK : u64 = 1;

const CARRIAGE_FLANK : u64 = FLANK;
const MILESTONE_CARRIAGE_FLANK : u64 = FLANK;

pub(crate) struct AbstractCarriageFactory {
    extent: TrainExtent,
    configs: TrainTrackConfigList,
    messages: MessageSender
}

impl AbstractCarriageFactory {
    pub(crate) fn new(extent: &TrainExtent, configs: &TrainTrackConfigList, messages: &MessageSender) -> AbstractCarriageFactory {
        AbstractCarriageFactory {
            extent: extent.clone(),
            configs: configs.clone(),
            messages: messages.clone()
        }
    }

    fn new_unloaded_carriage(&self, index: u64, ping_needed: &Needed) -> CarriageBuilder {
        CarriageBuilder::new(&CarriageExtent::new(&self.extent,index),Some(ping_needed),&self.configs,Some(&self.messages),false)
    }
}

pub(crate) struct AbstractTrainActions {
    data_api: PeregrineApiQueue,
    ping_needed: Needed,
    ready: bool,
    mute: bool,
    active: bool,
    carriage_factory: AbstractCarriageFactory,
    train_state_spec: TrainStateSpec,
    graphics: Graphics
}

impl AbstractTrainActions {
    pub(crate) fn new(data_api: &PeregrineApiQueue, ping_needed: &Needed, carriage_factory: AbstractCarriageFactory, 
            answer_allocator: &Arc<Mutex<AnswerAllocator>>,
            graphics: &Graphics) -> AbstractTrainActions {
        AbstractTrainActions {
            data_api: data_api.clone(),
            ping_needed: ping_needed.clone(),
            ready: false,
            mute: false,
            active: false,
            carriage_factory,
            graphics: graphics.clone(),
            train_state_spec: TrainStateSpec::new(answer_allocator)
        }
    }

    fn state_updated(&mut self) {
        if !self.mute && self.active {
            self.graphics.set_playing_field(self.state().playing_field());
            self.graphics.set_metadata(self.state().metadata());
        }
    }

    pub(crate) fn mute(&mut self, yn: bool) {
        self.mute = yn;
        if !self.mute {
            self.state_updated()
        }
    }

    pub(crate) fn active(&mut self) {
        if !self.active && !self.mute {
            self.active = true;
            self.state_updated();
        }
    }

    pub(crate) fn state(&self) -> TrainState3 { self.train_state_spec.spec() }
}

impl PartyActions<u64,CarriageBuilder,AbstractCarriage> for AbstractTrainActions {
    fn ctor(&mut self, index: &u64) -> CarriageBuilder {
        let new_carriage = self.carriage_factory.new_unloaded_carriage(*index,&self.ping_needed);
        #[cfg(debug_trains)] log!("CP ctor ({})",new_carriage.extent().compact());
        self.data_api.load_carriage(&new_carriage);
        new_carriage
    }

    fn dtor_pending(&mut self, index: &u64, _carriage: CarriageBuilder) {
        #[cfg(debug_trains)] log!("CP dtor_pending ({})",_carriage.extent().compact());
        self.train_state_spec.remove(*index);
        self.state_updated();
        self.ping_needed.set(); /* Need to call ping in case dc are ready */
    }

    fn dtor(&mut self, index: &u64, _carriage: AbstractCarriage) {
        #[cfg(debug_trains)] log!("CP dtor ({:?})",_carriage.extent().map(|x| x.compact()));
        self.train_state_spec.remove(*index);
        self.state_updated();
        self.ping_needed.set(); /* Need to call ping in case dc are ready */
    }

    fn init(&mut self, index: &u64, carriage: &mut CarriageBuilder) -> Option<AbstractCarriage> {
        carriage.get_carriage_output().map(|shapes| {
            #[cfg(debug_trains)] log!("CP init ({:?})",carriage.extent().compact());
            self.train_state_spec.add(*index,&shapes.spec().ok().unwrap()); // XXX errors
            self.state_updated();
            self.ping_needed.set(); /* Need to call ping in case dc are ready */
            shapes
        })
    }

    fn quiet(&mut self, _items: &mut dyn Iterator<Item=(&u64,&AbstractCarriage)>) {
        self.ready = true;
    }
}

pub struct AbstractTrain {
    party: Party<u64,CarriageBuilder,AbstractCarriage,AbstractTrainActions>,
    flank: u64,
    seen: PartyState
}

impl AbstractTrain {
    pub(crate) fn new(data_api: &PeregrineApiQueue, train_extent: &TrainExtent, ping_needed: &Needed, answer_allocator: &Arc<Mutex<AnswerAllocator>>, configs: &TrainTrackConfigList, graphics: &Graphics, messages: &MessageSender) -> AbstractTrain {
        let is_milestone = train_extent.scale().is_milestone();
        let carriage_factory = AbstractCarriageFactory::new(train_extent,configs,messages);
        let carriage_actions = AbstractTrainActions::new(data_api,ping_needed,carriage_factory,answer_allocator,graphics);
        AbstractTrain {
            party: Party::new(carriage_actions),
            flank: if is_milestone { MILESTONE_CARRIAGE_FLANK } else { CARRIAGE_FLANK },
            seen: PartyState::null()
        }
    }

    pub(crate) fn ping(&mut self) -> Option<(TrainState3,Vec<AbstractCarriage>)> {
        self.party.ping();
        if self.party.is_ready() && self.party.state() != self.seen {
            /* process was updated so update drawing target */
            let state = self.party.inner().state();
            let wanted = self.party.iter().map(|(_,x)| 
                x.clone()
            ).collect::<Vec<_>>();
            #[cfg(debug_trains)] debug_log!("CP->DP {}",wanted.iter().map(|x| x.extent().compact()).collect::<Vec<_>>().join(", "));
            self.seen = self.party.state(); 
            Some((state,wanted))
        } else {
            None
        }
    }

    pub(crate) fn is_ready(&self) -> bool { self.party.is_ready() }
    pub(crate) fn mute(&mut self, yn: bool) { self.party.inner_mut().mute(yn); }
    pub(crate) fn active(&mut self) { self.party.inner_mut().active(); }

    pub(crate) fn update_centre(&mut self, centre: u64) {
        let start = max((centre as i64)-(self.flank as i64),0) as u64;
        let wanted = start..(start+self.flank*2+1);    
        self.party.set(wanted);
    }
}
