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
    index: u64,
    ping_needed: Needed,
    extent: CarriageExtent,
    shapes: CarriageOutput
}

impl PartialEq for DrawingCarriageCreator {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl Eq for DrawingCarriageCreator {}

impl std::hash::Hash for DrawingCarriageCreator {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl DrawingCarriageCreator {
    fn create(&self, train_state: &TrainState3) -> DrawingCarriage2 {
        DrawingCarriage2::new(&self.extent,&self.ping_needed,&self.shapes,train_state)
    }
}

struct DrawingCarriages2 {
    ping_needed: Needed,
    train_extent: TrainExtent,
    graphics: Graphics
}

impl DrawingCarriages2 {
    fn new(ping_needed: &Needed, train_extent: &TrainExtent, graphics: &Graphics) -> DrawingCarriages2 {
        DrawingCarriages2 {
            ping_needed: ping_needed.clone(),
            train_extent: train_extent.clone(),
            graphics: graphics.clone()
        }
    }
}

impl SliderActions<(DrawingCarriageCreator,TrainState3),DrawingCarriage2,DrawingCarriage2> for DrawingCarriages2 {
    fn ctor(&mut self, (creator,state): &(DrawingCarriageCreator,TrainState3)) -> DrawingCarriage2 {
        debug_log!("create carriage {:?}",creator.index);
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
        let carriages = items.map(|x| x.1).cloned().collect::<Vec<_>>();
        if carriages.len() > 0 {
            self.graphics.set_carriages(&self.train_extent,&carriages);
        }
    }

    fn dtor(&mut self, _: &(DrawingCarriageCreator,TrainState3), dc: DrawingCarriage2) {
        self.graphics.drop_carriage(&dc);
    }
}

struct CarriageProcessActions2 {
    ping_needed: Needed,
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
            constant: constant.clone(),
            graphics: graphics.clone(),
            railway_data_tasks: railway_data_tasks.clone(),
            train_state_spec: TrainStateSpec::new(answer_allocator)
        }
    }

    fn state_updated(&mut self) {
        self.graphics.set_playing_field(self.state().playing_field());
        self.graphics.set_metadata(self.state().metadata());
    }

    fn state(&self) -> TrainState3 { self.train_state_spec.spec() }
}

impl SliderActions<u64,CarriageProcess,DrawingCarriageCreator> for CarriageProcessActions2 {
    fn ctor(&mut self, index: &u64) -> CarriageProcess {
        debug_log!("create panel {:?}",index);
        let new_carriage = self.constant.new_unloaded_carriage(*index);
        self.railway_data_tasks.add_carriage(&new_carriage);
        new_carriage
    }

    fn dtor(&mut self, index: &u64, _item: DrawingCarriageCreator) {
        self.train_state_spec.remove(*index);
        self.state_updated();
    }

    fn init(&mut self, index: &u64, item: &mut CarriageProcess) -> Option<DrawingCarriageCreator> {
        debug_log!("init panel? {:?}",index);
        item.get_shapes2().map(|shapes| {
            debug_log!("init panel! {:?}",index);
            self.train_state_spec.add(*index,shapes.spec());
            self.state_updated();
            self.ping_needed.set(); /* Need to call ping in case dc are ready */
            DrawingCarriageCreator { 
                index: *index,
                extent: item.extent().clone(),
                shapes: shapes.clone(),
                ping_needed: self.ping_needed.clone()
            }
        })
    }
}

pub(super) struct CarriageSet {
    centre: Option<u64>,
    active: bool,
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
            milestone: is_milestone,
            active: false
        }
    }

    pub(super) fn set_active(&mut self, yn: bool) {
        self.active = yn;
        if yn {
            self.ping();
        }
    }

    pub(super) fn is_active(&self) -> bool { self.active }

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

    pub(super) fn each_current_drawing_carriage<X,F>(&self, state: &mut X, mut cb: F) where F: FnMut(&mut X,&DrawingCarriage2) {
        for (_,drawing) in self.drawing.iter() {
            cb(state,drawing);
        }
    }

    pub(super) fn all_ready(&self) -> bool {
        /* for efficiency */
        if !self.process.is_ready() || !self.drawing.is_ready() { return false; }
        /**/
        let mut wanted = self.process.wanted().clone();
        for got in self.drawing.iter().map(|((x,_),_)| x.index) {
            wanted.remove(&got);
        }
        wanted.len() == 0
    }

    // TODO "good enough" layer via trains
    pub(super) fn ping(&mut self) {
        debug_log!("carriage_set/ping");
        self.process.check();
        /* Create any necessary DrawingCarriages */
        let state = self.process.inner().state();
        debug_log!("carriage_set/ping (active)");
        let mut wanted = self.process.iter().map(|(_,x)| (x.clone(),state.clone())).collect::<Vec<_>>();
        debug_log!("wanted len={}",wanted.len());
        self.drawing.set(&mut wanted.drain(..));
        /* Maybe we need to update the UI? */
        self.drawing.check();
    }
}
