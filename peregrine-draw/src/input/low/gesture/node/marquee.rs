use std::sync::Arc;
use peregrine_toolkit::{lock};
use crate::{shape::spectres::{stain::Stain, ants::MarchingAnts}, input::low::{lowlevel::LowLevelState, gesture::core::{finger::{OneOrTwoFingers, OneFinger}, transition::GestureNodeTransition, gesture::GestureNodeState, gesturenode::GestureNodeImpl}, pointer::PointerAction }, Message, run::CursorCircumstance };

pub(crate) struct Marquee {
    ants: Arc<MarchingAnts>,
    stain: Arc<Stain>
}

impl Marquee {
    fn make_ants(primary: &OneFinger) -> (f64,f64,f64,f64) {
        let pos = (
            primary.start().1,
            primary.start().0,
            primary.current().1,
            primary.current().0
        );
        (
            pos.0.min(pos.2),
            pos.1.min(pos.3),
            pos.0.max(pos.2),
            pos.1.max(pos.3)
        )
    }

    pub(super) fn new(lowlevel: &mut LowLevelState, fingers: &OneOrTwoFingers) -> Result<Marquee,Message> {
        let tlbr = Self::make_ants(fingers.primary());
        let ants = lowlevel.spectre_manager_mut().marching_ants()?;
        let stain = lowlevel.spectre_manager_mut().stain(true)?;
        ants.set_position(tlbr);
        stain.set_position(tlbr);
        Ok(Marquee { ants, stain })
    }

    fn update_spectres(&mut self, state: &mut GestureNodeState, primary: &OneFinger) -> Result<(),Message> {
        let tlbr = Self::make_ants(primary);
        self.ants.set_position(tlbr);
        self.stain.set_position(tlbr);    
        state.lowlevel.spectre_manager().update(&*lock!(state.gl))?;
        Ok(())
    }

    fn compute_hold(&self, state: &GestureNodeState, primary: &OneFinger) -> Result<Option<(f64,f64,f64)>,Message> {
        let pos_a = primary.start();
        let pos_b = primary.current();
        let (a,_,c,_) = (pos_a.0.min(pos_b.0),pos_a.1.min(pos_b.1),
                                              pos_a.0.max(pos_b.0),pos_a.1.max(pos_b.1));
        if let Some(stage) = state.lowlevel.stage() {
            let converter = stage.x().unit_converter()?;
            let want_bp_per_screen = converter.px_delta_to_bp(c-a);
            let centroid_bp = converter.px_pos_to_bp((c+a)/2.);
            if converter.delta_bp_to_px(want_bp_per_screen) < state.config.min_hold_drag_size {
                return Ok(None);
            }
            Ok(Some((want_bp_per_screen,centroid_bp,0.)))
        } else {
            Ok(None)
        }
    }
}

impl GestureNodeImpl for Marquee {
    fn init(&mut self, transition: &mut GestureNodeTransition, _state: &mut GestureNodeState, _fingers: &mut OneOrTwoFingers) -> Result<(),Message> {
        transition.set_cursor(CursorCircumstance::Hold);
        Ok(())
    }

    fn continues(&mut self, _transition: &mut GestureNodeTransition, state: &mut GestureNodeState, fingers: &mut OneOrTwoFingers) -> Result<(),Message> {
        let delta = fingers.primary_mut().take_delta();
        PointerAction::RunningHold(state.initial_modifiers.clone(),delta).emit(&state.lowlevel,true);
        self.update_spectres(state,fingers.primary())?;
        Ok(())
    }

    fn finished(&mut self, state: &mut GestureNodeState, fingers: &mut OneOrTwoFingers) -> Result<bool,Message> {
        let delta = fingers.primary_mut().take_delta();
        PointerAction::RunningHold(state.initial_modifiers.clone(),delta).emit(&state.lowlevel,false);
        if let Some((scale,centre,y)) = self.compute_hold(state,fingers.primary())? {
            PointerAction::HoldDrag(state.initial_modifiers.clone(),scale,centre,y).emit(&state.lowlevel,true);
        }
        Ok(true)
    }
}
