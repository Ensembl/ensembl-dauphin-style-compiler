use std::sync::Arc;
use peregrine_toolkit::lock;
use crate::{Message, run::CursorCircumstance, input::low::{gesture::core::{transition::GestureNodeTransition, finger::OneOrTwoFingers, gesture::GestureNodeState, gesturenode::GestureNodeImpl}, pointer::PointerAction, lowlevel::LowLevelState}, shape::spectres::maypole::Maypole};

pub(crate) struct MaypoleNode {
    maypole: Arc<Maypole>
}

impl MaypoleNode {
    pub(super) fn new(lowlevel: &mut LowLevelState, fingers: &OneOrTwoFingers) -> Result<MaypoleNode,Message> {
        let maypole = lowlevel.spectre_manager_mut().maypole()?;
        maypole.set_position(fingers.primary().current().0);
        Ok(MaypoleNode {
            maypole
        })
    }
}

impl GestureNodeImpl for MaypoleNode {
    fn init(&mut self, transition: &mut GestureNodeTransition, _state: &mut GestureNodeState, _fingers: &mut OneOrTwoFingers) -> Result<(),Message> {
        transition.set_cursor(CursorCircumstance::Maypole);
        Ok(())
    }

    fn continues(&mut self, _transition: &mut GestureNodeTransition, state: &mut GestureNodeState, fingers: &mut OneOrTwoFingers) -> Result<(),Message> {
        self.maypole.set_position(fingers.primary().current().0);
        state.lowlevel.spectre_manager().update(&*lock!(state.gl))?;
        Ok(())
    }

    fn finished(&mut self, state: &mut GestureNodeState, _fingers: &mut OneOrTwoFingers) -> Result<bool,Message> {
        Ok(true)
    }
}
