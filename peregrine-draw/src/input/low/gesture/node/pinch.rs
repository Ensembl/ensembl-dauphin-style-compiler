use crate::{Message, stage::{axis::UnitConverter, stage::ReadStage}, input::low::{gesture::core::{finger::{TwoFingers, OneOrTwoFingers}, transition::GestureNodeTransition, gesture::GestureNodeState, gesturenode::GestureNodeImpl}, pointer::{PointerConfig, PointerAction}}};

/* our x co-ordinates are measued in px. When we zoom out they reduce, ie zoom out has scale < 1.
 * this is the RECIPROCAL of the change in bp-per-screen used elsewhere.
 */
fn pixel_scale(fp: &TwoFingers, min_sep: f64) -> Option<f64> {
    let start_separation = fp.start_separation().0.abs();
    let current_separation = fp.current_separation().0.abs();
    if current_separation < min_sep { return None; }
    Some(current_separation/start_separation)
}

/* regular px_per_screen type scale, ie reciprocal of pixel_scale. Needs separate function
 * because different asymptote
 */
fn bp_scale(fp: &TwoFingers, min_sep: f64) -> Option<f64> {
    let start_separation = fp.start_separation().0.abs();
    let current_separation = fp.current_separation().0.abs();
    if current_separation < min_sep { return None; }
    Some(start_separation/current_separation)
}

/* the eigenpoint is the position (in pixels) which hasn't moved.
 * 
 */
fn eigenpoint(fp: &TwoFingers, min_sep: f64, min_scale: f64) -> Option<f64> {
    let scale = pixel_scale(fp,min_sep);
    if scale.is_none() { return None; }
    let scale = scale.unwrap();
    if (scale-1.).abs() < min_scale { return None; }
    let offset = fp.first().current().0 - fp.first().start().0 * scale;
    Some(offset/(1.-scale))
}

#[derive(Clone)]
pub(crate) struct PixelPinchAction {
    scale: f64,
    eigenpoint: f64,
    delta_y: f64
}

pub(crate) struct ScreenPosition {
    converter: UnitConverter,
    y_pos: f64
}

impl ScreenPosition {
    pub(crate) fn new(stage: &ReadStage) -> Result<ScreenPosition,Message> {
        let y = stage.y();
        Ok(ScreenPosition {
            converter: stage.x().unit_converter()?,
            y_pos: y.position()?
        })
    }

    pub(crate) fn transform(start: &ScreenPosition, action: &PixelPinchAction) -> ScreenPosition {
        let eigenpoint_in_screenfuls = start.converter.px_pos_to_screen_prop(action.eigenpoint);
        let eigenpoint_in_bp = start.converter.px_pos_to_bp(action.eigenpoint);
        let resized_converter = start.converter.resize_prop(action.scale);
        let eigenpoint_in_bp_from_centre = resized_converter.canvas_prop_to_bp_from_centre(eigenpoint_in_screenfuls);
        let centre_bp = eigenpoint_in_bp - eigenpoint_in_bp_from_centre;
        ScreenPosition {
            converter: resized_converter.move_to(centre_bp),
            y_pos: start.y_pos + action.delta_y
        }
    }
}

impl ScreenPosition {
    pub(crate) fn parameters(&self) -> Vec<f64> {
        vec![self.converter.bp_per_screen(),self.converter.position(),self.y_pos]
    }
}

pub(crate) struct Pinch {
    fingers: TwoFingers,
    best_pinch: PixelPinchAction,
    initial_screen: ScreenPosition,
    min_sep: f64,
    min_scale: f64
}

impl Pinch {
    pub(super) fn new(stage: &ReadStage, fingers: &TwoFingers, config: &PointerConfig) -> Result<Option<Pinch>,Message> {
        if !stage.ready() { return Ok(None); }
        Ok(Some(Pinch {
            fingers: fingers.clone(),
            best_pinch: PixelPinchAction { scale: 1., eigenpoint: 0., delta_y: 0. },
            initial_screen: ScreenPosition::new(stage)?,
            min_sep:  config.pinch_min_sep,
            min_scale: config.pinch_min_scale
        }))
    }

    fn make_pinch_action(&self, min_sep: f64, min_scale: f64) -> Option<PixelPinchAction> {
        let scale = bp_scale(&self.fingers,min_sep);
        let eigenpoint = eigenpoint(&self.fingers,min_sep,min_scale);
        let delta_y = self.fingers.start_mean().1 - self.fingers.current_mean().1;
        if let (Some(scale),Some(eigenpoint)) = (scale,eigenpoint) {
            Some(PixelPinchAction { scale, eigenpoint, delta_y })
        } else {
            None
        }
    }

    pub(crate) fn set_position(&mut self, primary: (f64,f64), secondary: (f64,f64)) {
        self.fingers.set_position(primary,secondary);
        if let Some(best_pinch) = self.make_pinch_action(self.min_sep,self.min_scale) {
            self.best_pinch = best_pinch;
        }
    }

    pub(crate) fn position(&self) -> ScreenPosition { ScreenPosition::transform(&self.initial_screen,&self.best_pinch) }
}

impl GestureNodeImpl for Pinch {
    fn continues(&mut self, _transition: &mut GestureNodeTransition, state: &mut GestureNodeState, fingers: &mut OneOrTwoFingers) -> Result<(),Message> {
        if let Some(two) = fingers.two_mut() {
            self.set_position(two.first().current(),two.second().current());
            PointerAction::RunningPinch(state.initial_modifiers.clone(),self.position()).emit(&state.lowlevel,true);
        }
        Ok(())
    }

    fn finished(&mut self, state: &mut GestureNodeState, _fingers: &mut OneOrTwoFingers) -> Result<bool,Message> {
        PointerAction::RunningPinch(state.initial_modifiers.clone(),self.position()).emit(&state.lowlevel,false);
        PointerAction::PinchDrag(state.initial_modifiers.clone(),self.position()).emit(&state.lowlevel,true);
        Ok(true)
    }
}
