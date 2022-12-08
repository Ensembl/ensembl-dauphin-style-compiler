use crate::{ Message, input::low::{gesture::{core::{transition::{GestureNodeTransition, TimerHandle}, gesture::{GestureNodeState}, finger::OneOrTwoFingers, gesturenode::{GestureNode, GestureNodeImpl}}, node::maypolenode::MaypoleNode}, pointer::PointerAction}, run::CursorCircumstance};
use super::{drag::Drag, commontools::{check_for_pinch, go_vertical}, marquee::Marquee, vertical::Vertical};

pub(crate) struct Unknown {
    vertical_done: bool,
    hold_timer: Option<TimerHandle>,
    cursor_timer: Option<TimerHandle>
}

impl Unknown {
    pub(crate) fn new() -> Unknown {
        Unknown {
            vertical_done: false,
            hold_timer: None,
            cursor_timer: None
        }
    }
}

impl GestureNodeImpl for Unknown {
    fn init(&mut self, transition: &mut GestureNodeTransition, state: &mut GestureNodeState, fingers: &mut OneOrTwoFingers) -> Result<(),Message> {
        let maypole = state.lowlevel.special_status(|s| {
            s.iter().filter(|x| {
                x.name == "maypole"
            }).cloned().next()
        });
        if let Some(maypole) = maypole {
            transition.new_mode(GestureNode::new(MaypoleNode::new(&mut state.lowlevel,&fingers,&maypole)?));
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
        let (vertical,too_far) = go_vertical(fingers,&state.config);
        if !self.vertical_done && vertical {
            transition.new_mode(GestureNode::new(Vertical::new()));
            return Ok(());
        }
        self.vertical_done = too_far;
        if fingers.primary().total_distance() > state.config.click_radius {
            transition.new_mode(GestureNode::new(Drag::new(too_far)));
        }
        Ok(())
    }

    fn finished(&mut self, _state: &mut GestureNodeState, _fingers: &mut OneOrTwoFingers) -> Result<bool,Message> {
        Ok(false)
    }
}
