use crate::Message;

use super::{finger::{ OneOrTwoFingers}, gesture::{GestureNodeState}, transition::{GestureNodeTransition, TimerHandle}};

pub(crate) trait GestureNodeImpl {
    fn init(&mut self, _transition: &mut GestureNodeTransition, _state: &mut GestureNodeState, _fingers: &mut OneOrTwoFingers) -> Result<(),Message> { Ok(()) }
    fn timeout(&mut self, _transition: &mut GestureNodeTransition, _state: &mut GestureNodeState, _fingers: &mut OneOrTwoFingers, _handle: TimerHandle) -> Result<(),Message> { Ok(()) }
    fn continues(&mut self, transition: &mut GestureNodeTransition, state: &mut GestureNodeState, fingers: &mut OneOrTwoFingers) -> Result<(),Message>;
    fn finished(&mut self, state: &mut GestureNodeState, fingers: &mut OneOrTwoFingers) -> Result<bool,Message>;
}

pub(crate) struct GestureNode(Box<dyn GestureNodeImpl>);

impl GestureNode {
    pub(crate) fn new<F>(imp: F) -> GestureNode where F: GestureNodeImpl + 'static {
        GestureNode(Box::new(imp))
    }
}

impl GestureNodeImpl for GestureNode {
    fn init(&mut self, transition: &mut GestureNodeTransition, state: &mut GestureNodeState, fingers: &mut OneOrTwoFingers) -> Result<(),Message> {
        self.0.init(transition,state,fingers)
    }

    fn timeout(&mut self, transition: &mut GestureNodeTransition, state: &mut GestureNodeState,fingers: &mut OneOrTwoFingers, handle: TimerHandle) -> Result<(),Message> {
        self.0.timeout(transition,state,fingers,handle)
    }

    fn continues(&mut self, transition: &mut GestureNodeTransition, state: &mut GestureNodeState, fingers: &mut OneOrTwoFingers) -> Result<(),Message> {
        self.0.continues(transition,state,fingers)
    }

    fn finished(&mut self,state: &mut GestureNodeState, fingers: &mut OneOrTwoFingers) -> Result<bool,Message> {
        self.0.finished(state,fingers)
    }
}
