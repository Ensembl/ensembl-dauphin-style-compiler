use peregrine_toolkit::{time::now, rate::Rate};

use crate::{input::low::{gesture::core::{gesturenode::GestureNodeImpl, transition::GestureNodeTransition, gesture::GestureNodeState, finger::OneOrTwoFingers}, pointer::PointerAction}, Message, run::CursorCircumstance};

use super::commontools::check_for_pinch;

pub(crate) struct Vertical {
    prev_sample: Option<f64>,
    rate: Rate
}

impl Vertical {
    pub(crate) fn new() -> Vertical { 
        Vertical {
            prev_sample: None,
            rate: Rate::new(100.)
        }
    }
}

impl GestureNodeImpl for Vertical {
    fn init(&mut self, transition: &mut GestureNodeTransition, state: &mut GestureNodeState, fingers: &mut OneOrTwoFingers) -> Result<(),Message> {
        transition.set_cursor(CursorCircumstance::Vertical);
        Ok(())
    }

    fn continues(&mut self, transition: &mut GestureNodeTransition, state: &mut GestureNodeState, fingers: &mut OneOrTwoFingers) -> Result<(),Message> {
        if check_for_pinch(transition,state,fingers)? { return Ok(()); }
        let mut delta = fingers.primary_mut().take_delta();
        let now = now();
        if let Some(stored) = &mut self.prev_sample {
            let delta_ms = now - *stored;
            self.rate.add_sample(delta_ms,delta.1.abs());
        }
        self.prev_sample = Some(now);
        let rate = (self.rate.sample().unwrap_or(0.)/100.).max(1.).min(10.);
        delta.1 = delta.1 * rate;
        PointerAction::VerticalDrag(state.initial_modifiers.clone(),delta).emit(&state.lowlevel,true);
        Ok(())
    }

    fn finished(&mut self, _state: &mut GestureNodeState, _fingers: &mut OneOrTwoFingers) -> Result<bool,Message> {
        Ok(true)
    }
}
