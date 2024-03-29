use crate::{run::CursorCircumstance, Message, input::low::{pointer::{PointerAction, PointerConfig}, gesture::core::{gesture::GestureNodeState, transition::GestureNodeTransition, gesturenode::GestureNode, finger::OneOrTwoFingers}}};
use super::pinch::Pinch;

pub(super) fn check_for_pinch(transition: &mut GestureNodeTransition, state: &mut GestureNodeState, fingers: &mut OneOrTwoFingers) -> Result<bool,Message> {
    let was_upgraded = fingers.take_upgraded();
    let mut new_pinch = None;
    if let Some(two) = &mut fingers.two() {
        if was_upgraded {
            if let Some(stage) = state.lowlevel.stage() {
                if let Some(pinch) = Pinch::new(&stage,two,&state.config)? {
                    new_pinch = Some(pinch);
                }
            }
        }
    }
    Ok(if let Some(pinch) = new_pinch {
        let position = pinch.position();
        PointerAction::SwitchToPinch(state.initial_modifiers.clone(),position).emit(&state.lowlevel,true);
        transition.new_mode(GestureNode::new(pinch));
        transition.set_cursor(CursorCircumstance::Pinch);
        true
    } else {
        false
    })
}

pub(super) fn go_vertical(fingers: &OneOrTwoFingers, config: &PointerConfig) -> (bool,bool) {
    let odometer = fingers.primary().odometer();
    let total_px = fingers.primary().total_delta();
    ((total_px.0+total_px.1).abs() > config.min_vert_denom && 
                  total_px.1.abs() > config.min_vert_numer,
      odometer.0+odometer.1 > config.min_vert_odometer)
}
