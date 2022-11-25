use crate::{Message, run::CursorCircumstance, input::low::{gesture::core::{transition::GestureNodeTransition, finger::OneOrTwoFingers, gesture::GestureNodeState, gesturenode::GestureNodeImpl}, pointer::PointerAction}};
use super::{commontools::check_for_pinch};

pub(crate) struct Drag;

impl Drag {
    pub(super) fn new() -> Drag { Drag }
}

impl GestureNodeImpl for Drag {
    fn new(&mut self, transition: &mut GestureNodeTransition, _state: &mut GestureNodeState, _fingers: &mut OneOrTwoFingers) -> Result<(),Message> {
        transition.set_cursor(CursorCircumstance::Drag);
        Ok(())
    }

    fn continues(&mut self, transition: &mut GestureNodeTransition, state: &mut GestureNodeState, fingers: &mut OneOrTwoFingers) -> Result<(),Message> {
        if check_for_pinch(transition,state,fingers)? { return Ok(()); }
        let delta = fingers.primary_mut().take_delta();
        PointerAction::RunningDrag(state.initial_modifiers.clone(),delta).emit(&state.lowlevel,true);
        Ok(())
    }

    fn finished(&mut self, state: &mut GestureNodeState, fingers: &mut OneOrTwoFingers) -> Result<bool,Message> {
        let delta = fingers.primary_mut().take_delta();
        let total_delta = fingers.primary().total_delta();
        PointerAction::RunningDrag(state.initial_modifiers.clone(),delta).emit(&state.lowlevel,false);
        PointerAction::Drag(state.initial_modifiers.clone(),total_delta).emit(&state.lowlevel,true);
        Ok(true)
    }
}
