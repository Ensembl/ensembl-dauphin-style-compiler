use std::cmp::max;
use std::sync::{Mutex, Arc};
use peregrine_toolkit::{lock, log, debug_log};
use peregrine_toolkit::puzzle::AnswerAllocator;
use peregrine_toolkit::sync::needed::Needed;

use super::drawingcarriage::DrawingCarriage2;
use super::graphics::Graphics;
use super::railwaydatatasks::RailwayDataTasks;
use super::slider::{Slider, SliderActions};
use super::trainextent::TrainExtent;
use crate::allotment::core::carriageoutput::CarriageOutput;
use crate::allotment::core::trainstate::{TrainStateSpec, TrainState3};
use crate::shapeload::carriageprocess::CarriageProcess;
use crate::{CarriageExtent};
use crate::api::MessageSender;
use crate::switch::trackconfiglist::TrainTrackConfigList;

const CARRIAGE_FLANK : u64 = 1;
const MILESTONE_CARRIAGE_FLANK : u64 = 1;

struct CarriageSetConstant {
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

    fn new_unloaded_carriage(&self, index: u64) -> CarriageProcess {
        CarriageProcess::new(&CarriageExtent::new(&self.extent,index),Some(&self.ping_needed),&self.configs,Some(&self.messages),false)
    }
}

#[derive(Clone)]
struct DrawingCarriageCreator {
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
    fn create(&self, train_state: &TrainState3) -> DrawingCarriage2 {
        DrawingCarriage2::new(&self.extent,&self.ping_needed,&self.shapes,train_state)
            .ok().unwrap() // XXX errors
    }
}

struct DrawingCarriages2 {
    active: bool,
    ping_needed: Needed,
    train_extent: TrainExtent,
    carriages: Vec<DrawingCarriage2>,
    graphics: Graphics
}

impl DrawingCarriages2 {
    fn new(ping_needed: &Needed, train_extent: &TrainExtent, graphics: &Graphics) -> DrawingCarriages2 {
        DrawingCarriages2 {
            active: false,
            ping_needed: ping_needed.clone(),
            train_extent: train_extent.clone(),
            carriages: vec![],
            graphics: graphics.clone()
        }
    }

    fn send_carriages(&self) {
        if self.active && self.carriages.len() > 0 {
            self.graphics.set_carriages(&self.train_extent,&self.carriages);
        }
    }

    fn is_active(&self) -> bool { self.active }

    fn set_active(&mut self) {
        self.active = true;
        self.send_carriages();
    }
}

impl SliderActions<(DrawingCarriageCreator,TrainState3),DrawingCarriage2,DrawingCarriage2> for DrawingCarriages2 {
    fn ctor(&mut self, (creator,state): &(DrawingCarriageCreator,TrainState3)) -> DrawingCarriage2 {
        #[cfg(debug_trains)] debug_log!("create dc {:?}",creator.extent);
        let dc = creator.create(state);
        self.graphics.create_carriage(&dc);
        dc
    }

    fn init(&mut self, _: &(DrawingCarriageCreator,TrainState3), item: &mut DrawingCarriage2) -> Option<DrawingCarriage2> {
        if !item.is_ready() { return None; }
        self.ping_needed.set(); // train can maybe be updates
        Some(item.clone())
    }

    fn done(&mut self, items: &mut dyn Iterator<Item=(&(DrawingCarriageCreator, TrainState3), &DrawingCarriage2)>) {
        self.carriages = items.map(|x| x.1).cloned().collect::<Vec<_>>();
        #[cfg(debug_trains)] debug_log!("set dcs {:?}",self.carriages.iter().map(|x| x.extent()).collect::<Vec<_>>());
        self.send_carriages();
    }

    fn dtor(&mut self, (dcc,_): &(DrawingCarriageCreator,TrainState3), dc: DrawingCarriage2) {
        #[cfg(debug_trains)] debug_log!("drop dc {:?}",dcc.extent);
        self.graphics.drop_carriage(&dc);
    }
}

struct CarriageProcessActions2 {
    ping_needed: Needed,
    mute: bool,
    constant: Arc<CarriageSetConstant>,
    railway_data_tasks: RailwayDataTasks, 
    train_state_spec: TrainStateSpec,
    graphics: Graphics
}

impl CarriageProcessActions2 {
    fn new(ping_needed: &Needed, constant: &Arc<CarriageSetConstant>, 
           railway_data_tasks: &RailwayDataTasks, answer_allocator: &Arc<Mutex<AnswerAllocator>>,
            graphics: &Graphics) -> CarriageProcessActions2 {
        CarriageProcessActions2 {
            ping_needed: ping_needed.clone(),
            mute: false,
            constant: constant.clone(),
            graphics: graphics.clone(),
            railway_data_tasks: railway_data_tasks.clone(),
            train_state_spec: TrainStateSpec::new(answer_allocator)
        }
    }

    fn state_updated(&mut self) {
        if !self.mute {
            self.graphics.set_playing_field(self.state().playing_field());
            self.graphics.set_metadata(self.state().metadata());
        }
    }

    fn mute(&mut self, yn: bool) {
        self.mute = yn;
        if !self.mute {
            self.state_updated()
        }
    }

    fn state(&self) -> TrainState3 { self.train_state_spec.spec() }
}

impl SliderActions<u64,CarriageProcess,DrawingCarriageCreator> for CarriageProcessActions2 {
    fn ctor(&mut self, index: &u64) -> CarriageProcess {
        let new_carriage = self.constant.new_unloaded_carriage(*index);
        self.railway_data_tasks.add_carriage(&new_carriage);
        new_carriage
    }

    fn dtor(&mut self, index: &u64, _item: DrawingCarriageCreator) {
        self.train_state_spec.remove(*index);
        self.state_updated();
    }

    fn init(&mut self, index: &u64, item: &mut CarriageProcess) -> Option<DrawingCarriageCreator> {
        item.get_shapes2().map(|shapes| {
            self.train_state_spec.add(*index,&shapes.spec().ok().unwrap()); // XXX errors
            self.state_updated();
            self.ping_needed.set(); /* Need to call ping in case dc are ready */
            DrawingCarriageCreator { 
                //index: *index,
                extent: item.extent().clone(),
                shapes: shapes.clone(),
                ping_needed: self.ping_needed.clone()
            }
        })
    }
}

pub(super) struct CarriageSet {
    centre: Option<u64>,
    milestone: bool,
    drawing: Slider<(DrawingCarriageCreator,TrainState3),DrawingCarriage2,DrawingCarriage2,DrawingCarriages2>,
    process: Slider<u64,CarriageProcess,DrawingCarriageCreator,CarriageProcessActions2>
}

impl CarriageSet {
    pub(super) fn new(ping_needed: &Needed, answer_allocator: &Arc<Mutex<AnswerAllocator>>, extent: &TrainExtent, configs: &TrainTrackConfigList, railway_data_tasks: &RailwayDataTasks, graphics: &Graphics, messages: &MessageSender) -> CarriageSet {
        let constant = Arc::new(CarriageSetConstant::new(ping_needed,extent,configs,messages));
        let carriage_actions = CarriageProcessActions2::new(ping_needed,&constant,railway_data_tasks,answer_allocator,graphics);
        let drawing_actions = DrawingCarriages2::new(&ping_needed,extent,graphics);
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
    }

    pub(super) fn activate(&mut self) {
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

    pub(super) fn central_drawing_carriage(&self) -> Option<&DrawingCarriage2> {
        let index = if let Some(x) = self.centre { x } else { return None; };
        let creator = if let Some(creator) = self.process.get(index) {creator } else { return None; }.clone();
        let state = self.process.inner().state();
        self.drawing.get((creator,state))
    }

    pub(super) fn all_ready(&self) -> bool {
        /* for efficiency */
        if !self.process.is_ready() || !self.drawing.is_ready() { return false; }
        /**/
        let mut wanted = self.process.wanted().clone();
        for got in self.drawing.iter().map(|((x,_),_)| x.extent.index()) {
            wanted.remove(&got);
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
