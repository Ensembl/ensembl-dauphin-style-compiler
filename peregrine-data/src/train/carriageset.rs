use std::cmp::max;
use std::sync::{Mutex, Arc};
use peregrine_toolkit::puzzle::AnswerAllocator;
use peregrine_toolkit::sync::needed::Needed;
use super::carriageextent::CarriageExtent;
use super::carriageprocessmanager::CarriageProcessManager;
use super::drawingcarriage::DrawingCarriage;
use super::drawingcarriagemanager::{DrawingCarriageCreator, DrawingCarriageManager, SliderDrawingCarriage};
use super::graphics::Graphics;
use super::railwaydatatasks::RailwayDataTasks;
use super::slider::{Slider};
use super::trainextent::TrainExtent;
use crate::allotment::core::trainstate::{TrainState3};
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
    drawing: Slider<(DrawingCarriageCreator,TrainState3),SliderDrawingCarriage,SliderDrawingCarriage,DrawingCarriageManager>,
    process: Slider<u64,CarriageProcess,DrawingCarriageCreator,CarriageProcessManager>
}

impl CarriageSet {
    pub(super) fn new(ping_needed: &Needed, answer_allocator: &Arc<Mutex<AnswerAllocator>>, extent: &TrainExtent, configs: &TrainTrackConfigList, railway_data_tasks: &RailwayDataTasks, graphics: &Graphics, messages: &MessageSender) -> CarriageSet {
        let constant = Arc::new(CarriageSetConstant::new(ping_needed,extent,configs,messages));
        let carriage_actions = CarriageProcessManager::new(ping_needed,&constant,railway_data_tasks,answer_allocator,graphics);
        let drawing_actions = DrawingCarriageManager::new(&ping_needed,extent,graphics);
        let is_milestone = extent.scale().is_milestone();
        CarriageSet {
            centre: None,
            drawing: Slider::new(drawing_actions),
            process: Slider::new(carriage_actions),
            milestone: is_milestone
        }
    }

    pub(super) fn mute(&mut self, yn: bool) {
        self.process.inner_mut().mute(yn);
        self.drawing.inner_mut().mute(yn);
    }

    pub(super) fn activate(&mut self) {
        self.process.inner_mut().active();
        self.drawing.inner_mut().set_active();
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
        let index = if let Some(x) = self.centre { x } else { return None; };
        let creator = if let Some(creator) = self.process.get(index) {creator } else { return None; }.clone();
        let state = self.process.inner().state();
        self.drawing.get((creator,state)).map(|x| x.carriage())
    }

    pub(super) fn all_ready(&self) -> bool {
        /* for efficiency */
        if !self.process.is_ready() || !self.drawing.is_ready() { return false; }
        /**/
        let current_state = self.process.inner().state();
        let mut wanted = self.process.wanted().clone();
        for (got,got_state) in self.drawing.iter()
                .map(|((dcc,c),_)| (dcc.extent().index(),c)) {
            wanted.remove(&got);
            if got_state != &current_state { return false; }
        }
        wanted.len() == 0
    }

    /* only update drawings when not yet active when all are ready */
    fn test_should_update_drawings(&self) -> bool {
        self.drawing.inner().is_active() ||
        self.process.is_ready()        
    }

    // TODO "good enough" layer via trains
    pub(super) fn ping(&mut self) {
        self.process.check();
        if self.test_should_update_drawings() {
            /* Create any necessary DrawingCarriages */
            let state = self.process.inner().state();
            let mut wanted = self.process.iter().map(|(_,x)| (x.clone(),state.clone())).collect::<Vec<_>>();
            self.drawing.set(&mut wanted.drain(..));
            /* Maybe we need to update the UI? */
            self.drawing.check();
        }
    }
}
