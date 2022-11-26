use crate::{ Message, input::low::{gesture::{core::{transition::{GestureNodeTransition, TimerHandle}, gesture::{GestureNodeState}, finger::OneOrTwoFingers, gesturenode::{GestureNode, GestureNodeImpl}}, node::maypolenode::MaypoleNode}, pointer::PointerAction, lowlevel}, run::CursorCircumstance};
use super::{drag::Drag, commontools::check_for_pinch, marquee::Marquee};

pub(crate) struct Unknown {
    hold_timer: Option<TimerHandle>,
    cursor_timer: Option<TimerHandle>
}

impl Unknown {
    pub(crate) fn new() -> Unknown {
        Unknown {
            hold_timer: None,
            cursor_timer: None
        }
    }
}

impl GestureNodeImpl for Unknown {
    fn init(&mut self, transition: &mut GestureNodeTransition, state: &mut GestureNodeState, fingers: &mut OneOrTwoFingers) -> Result<(),Message> {
        if state.lowlevel.special_status(|s| s.contains(&"maypole".to_string())) {
            transition.new_mode(GestureNode::new(MaypoleNode::new(&mut state.lowlevel,&fingers)?));
            return Ok(());
        }
        self.hold_timer = Some(transition.add_timer(state.config.hold_delay));
        self.cursor_timer = Some(transition.add_timer(state.config.drag_cursor_delay));
        Ok(())
    }

    fn timeout(&mut self, transition: &mut GestureNodeTransition, state: &mut GestureNodeState, fingers: &mut OneOrTwoFingers, handle: TimerHandle) -> Result<(),Message> {
        if let Some(hold_timer) = &self.hold_timer {
            if handle == *hold_timer && state.lowlevel.special_status(|s| s.len()) == 0 {
                transition.new_mode(GestureNode::new(Marquee::new(&mut state.lowlevel,fingers)?));
                PointerAction::SwitchToHold(state.initial_modifiers.clone(),fingers.primary().start()).emit(&state.lowlevel,true);
            }
        }
        if let Some(cursor_timer) = &self.cursor_timer {
            if handle == *cursor_timer {
                transition.set_cursor(CursorCircumstance::Drag);
            }
        }
        Ok(())
    }

    fn continues(&mut self, transition: &mut GestureNodeTransition, state: &mut GestureNodeState, fingers: &mut OneOrTwoFingers) -> Result<(),Message> {
        if check_for_pinch(transition,state,fingers)? { return Ok(()); }
        if fingers.primary().total_distance() > state.config.click_radius {
            transition.new_mode(GestureNode::new(Drag::new()));
        }
        Ok(())
    }

    fn finished(&mut self, _state: &mut GestureNodeState, _fingers: &mut OneOrTwoFingers) -> Result<bool,Message> {
        Ok(false)
    }
}
