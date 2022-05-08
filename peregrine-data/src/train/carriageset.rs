use std::cmp::max;
use std::sync::{Mutex, Arc};
use peregrine_toolkit::puzzle::AnswerAllocator;
use peregrine_toolkit::sync::needed::Needed;
use super::carriageextent::CarriageExtent;
use super::carriageprocessparty::CarriageProcessManager;
use super::drawingcarriage::DrawingCarriage;
use super::drawingcarriagemanager::{DrawingCarriageCreator };
use super::drawingcarriagemanager2::DrawingCarriageSwitcher;
use super::graphics::Graphics;
use super::railwaydatatasks::RailwayDataTasks;
use super::party::{Party, PartyState};
use super::trainextent::TrainExtent;
use crate::shapeload::carriageprocess::CarriageProcess;
use crate::api::MessageSender;
use crate::switch::trackconfiglist::TrainTrackConfigList;

const CARRIAGE_FLANK : u64 = 2;
const MILESTONE_CARRIAGE_FLANK : u64 = 2;

pub(super) struct CarriageSetConstant {
    ping_needed: Needed,
    extent: TrainExtent,
    configs: TrainTrackConfigList,
    messages: MessageSender
}

impl CarriageSetConstant {
    fn new(ping_needed: &Needed, extent: &TrainExtent, configs: &TrainTrackConfigList, messages: &MessageSender) -> CarriageSetConstant {
        CarriageSetConstant {
            ping_needed: ping_needed.clone(),
            extent: extent.clone(),
            configs: configs.clone(),
            messages: messages.clone()
        }
    }

    pub(super) fn new_unloaded_carriage(&self, index: u64) -> CarriageProcess {
        CarriageProcess::new(&CarriageExtent::new(&self.extent,index),Some(&self.ping_needed),&self.configs,Some(&self.messages),false)
    }
}

pub(super) struct CarriageSet {
    centre: Option<u64>,
    milestone: bool,
    drawing: DrawingCarriageSwitcher,
    process: Party<u64,CarriageProcess,DrawingCarriageCreator,CarriageProcessManager>,
    seen_process_partystate: PartyState
}

impl CarriageSet {
    pub(super) fn new(ping_needed: &Needed, answer_allocator: &Arc<Mutex<AnswerAllocator>>, extent: &TrainExtent, configs: &TrainTrackConfigList, railway_data_tasks: &RailwayDataTasks, graphics: &Graphics, messages: &MessageSender) -> CarriageSet {
        let constant = Arc::new(CarriageSetConstant::new(ping_needed,extent,configs,messages));
        let carriage_actions = CarriageProcessManager::new(ping_needed,&constant,railway_data_tasks,answer_allocator,graphics);
        let is_milestone = extent.scale().is_milestone();
        CarriageSet {
            centre: None,
            drawing: DrawingCarriageSwitcher::new(ping_needed,extent,graphics),
            process: Party::new(carriage_actions),
            milestone: is_milestone,
            seen_process_partystate: PartyState::null()
        }
    }

    pub(super) fn mute(&mut self, yn: bool) {
        self.process.inner_mut().mute(yn);
        if yn {
            self.drawing.set_mute();
        }
    }

    pub(super) fn activate(&mut self) {
        self.process.inner_mut().active();
        self.drawing.set_active();
        self.ping();
    }

    pub(super) fn update_centre(&mut self, centre: u64) {
        self.centre = Some(centre);
        let flank = if self.milestone { MILESTONE_CARRIAGE_FLANK } else { CARRIAGE_FLANK };
        let start = max((centre as i64)-(flank as i64),0) as u64;
        let wanted = start..(start+flank*2+1);    
        self.process.set(wanted);
    }

    pub(super) fn central_drawing_carriage(&self) -> Option<&DrawingCarriage> {
        self.drawing.central_carriage()
    }

    pub(super) fn all_ready(&self) -> bool {
        self.process.is_ready() && self.drawing.is_ready()
    }

    /* We need to do a lot in a ping but also make sure we don't customarily do too much!
     *
     * We need to update the CarriageProcess party to see if any more processes are
     * ready. If they are then partystate will have been updated. In this case we need to
     * transfer copied of these into the DrawingCarriageSwitcher.
     * 
     * We should always ping DrawingCarriageSwitcher anyway to update anything that's
     * changed there.
     * 
     * So we need to make sure ping is scheduled:
     * 1. After anything which could cause a readiness change in CarriageProcess. This is
     *    ensured by init and dtor in CarriageProcess and the set at the end of the
     *    CarriageProcess async method itself.
     * 2. After anything which could cause a readiness change in DrawingCarriageSwitcher.
     *    This would be caused by anything which could cause a readiness change in any of
     *    the underlying DrawingCarriagePartys. This is triggered by the set in set_ready
     *    and dtor. Switcher pings respond to changes in ready() in any of their objects.
     *    In this case, this is caused by calls to quiet(). Which gets called by ping in
     *    party.
     */
    pub(super) fn ping(&mut self) {
        self.process.ping();
        if self.process.is_ready() && self.process.state() != self.seen_process_partystate {
            /* process was updated so update drawing target */
            let state = self.process.inner().state();
            let wanted = self.process.iter().map(|(_,x)| 
                x.clone()
            ).collect::<Vec<_>>();
            self.drawing.set(&state,&wanted);
            self.seen_process_partystate = self.process.state(); 
        }
        self.drawing.ping();
    }
}

// TODO add logs
// TODO test
// TODO check for excess
// TODO remove unused code
// TODO lifecycle enum
