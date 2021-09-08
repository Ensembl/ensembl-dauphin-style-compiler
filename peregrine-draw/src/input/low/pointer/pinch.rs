use crate::{Message, stage::stage::ReadStage};

use super::pointer::PointerConfig;

#[derive(Copy,Clone)]
pub(super) struct FingerAxis { // XX not pub
    start: f64,
    current: f64
}

impl FingerAxis {
    pub(super) fn new(position: f64) -> FingerAxis {
        FingerAxis {
            start: position,
            current: position
        }
    }

    pub(super) fn start(&self) -> f64 { self.start }
    pub(super) fn current(&self) -> f64 { self.current }
    pub(super) fn set(&mut self, position: f64) { self.current = position; }
    pub(super) fn reset(&mut self) { self.start = self.current; }
    pub(super) fn delta(&self) -> f64 { self.current - self.start }
}

struct FingerPairAxis(FingerAxis,FingerAxis);

impl FingerPairAxis {
    fn new(primary: f64, secondary: f64) -> FingerPairAxis {
        FingerPairAxis(FingerAxis::new(primary),FingerAxis::new(secondary))
    }

    fn set_position(&mut self, primary: f64, secondary: f64) {
        let order_before = self.0.current() > self.1.current();
        let order_after = primary > secondary;
        if order_before != order_after {
            let t = self.0;
            self.0 = self.1;
            self.1 = t;
        }
        self.0.set(primary);
        self.1.set(secondary);
    }

    fn reset(&mut self) { self.0.reset(); self.1.reset(); }
    fn current_separation(&self) -> f64 { self.1.current() - self.0.current() }
    fn start_separation(&self) -> f64 { self.1.start() - self.0.start() }
    fn current_mean(&self) -> f64 { (self.1.current() + self.0.current())/2. }
    fn start_mean(&self) -> f64 { (self.1.start() - self.0.start())/2. }

    /* our x co-ordinates are measued in px. When we zoom out they reduce, ie zoom out has scale < 1.
     * this is the RECIPROCAL of the change in bp-per-screen used elsewhere.
     */
    fn pixel_scale(&self, min_sep: f64) -> Option<f64> {
        let start_separation = self.start_separation().abs();
        let current_separation = self.current_separation().abs();
        if current_separation < min_sep { return None; }
        Some(current_separation/start_separation)
    }

    /* regular px_per_screen type scale, ie reciprocal of pixel_scale. Needs separate function
     * because different asymptote
     */
    fn bp_scale(&self, min_sep: f64) -> Option<f64> {
        let start_separation = self.start_separation().abs();
        let current_separation = self.current_separation().abs();
        if current_separation < min_sep { return None; }
        Some(start_separation/current_separation)
    }

    fn eigenpoint(&self, min_sep: f64, min_scale: f64) -> Option<f64> {
        let scale = self.pixel_scale(min_sep);
        if scale.is_none() { return None; }
        let scale = scale.unwrap();
        if (scale-1.).abs() < min_scale { return None; }
        let offset = self.0.current() - self.0.start() * scale;
        Some(offset/(1.-scale))
    }
}

#[derive(Clone)]
pub(crate) struct PixelPinchAction {
    scale: f64,
    eigenpoint: f64,
    delta_y: f64
}

impl PixelPinchAction {
    pub(crate) fn parameters(&self) -> Vec<f64> {
        vec![self.scale,self.eigenpoint,self.delta_y]
    }
}

struct FingerPair(FingerPairAxis,FingerPairAxis);

impl FingerPair {
    fn new(primary: (f64,f64), secondary: (f64,f64)) -> FingerPair {
        FingerPair(FingerPairAxis::new(primary.0,secondary.0),
                   FingerPairAxis::new(primary.1,secondary.1))
    }

    fn set_position(&mut self, primary: (f64,f64), secondary: (f64,f64)) {
        self.0.set_position(primary.0,secondary.0);
        self.1.set_position(primary.1, secondary.1);
    }

    fn make_pinch_action(&self, min_sep: f64, min_scale: f64) -> Option<PixelPinchAction> {
        let scale = self.0.bp_scale(min_sep);
        let eigenpoint = self.0.eigenpoint(min_sep,min_scale);
        let delta_y = self.1.start_mean() - self.1.current_mean();
        if let (Some(scale),Some(eigenpoint)) = (scale,eigenpoint) {
            Some(PixelPinchAction { scale, eigenpoint, delta_y })
        } else {
            None
        }
    }
}

pub(crate) struct ScreenPosition {
    centre_bp: f64,
    bp_per_screen: f64,
    y_pos: f64,
    screen_x: f64
}

impl ScreenPosition {
    pub(crate) fn new(stage: &ReadStage) -> Result<ScreenPosition,Message> {
        let x = stage.x();
        let y = stage.y();
        Ok(ScreenPosition {
            centre_bp: x.position()?,
            bp_per_screen: x.bp_per_screen()?,
            y_pos: y.position()?,
            screen_x: x.drawable_size()?
        })
    }

    pub(crate) fn transform(start: &ScreenPosition, action: &PixelPinchAction) -> ScreenPosition {
        let bp_per_screen = start.bp_per_screen * action.scale;
        let eigenpoint_in_screenfuls = (action.eigenpoint / start.screen_x)-0.5; // -0.5=left, +0.5=right
        let eigenpoint_in_bp = start.centre_bp + eigenpoint_in_screenfuls * start.bp_per_screen;
        let eigenpoint_in_bp_from_centre = eigenpoint_in_screenfuls * bp_per_screen;
        let centre_bp = eigenpoint_in_bp - eigenpoint_in_bp_from_centre;
        ScreenPosition {
            centre_bp,
            bp_per_screen,
            y_pos: start.y_pos + action.delta_y,
            screen_x: start.screen_x
        }
    }
}

impl ScreenPosition {
    pub(crate) fn parameters(&self) -> Vec<f64> {
        vec![self.bp_per_screen,self.centre_bp,self.y_pos]
    }
}

pub(crate) struct PinchManager {
    fingers: FingerPair,
    best_pinch: PixelPinchAction,
    initial_screen: ScreenPosition,
    min_sep: f64,
    min_scale: f64
}

impl PinchManager {
    fn new(stage: &ReadStage, primary: (f64,f64), secondary: (f64,f64), min_sep: f64, min_scale: f64) -> Result<Option<PinchManager>,Message> {
        if !stage.ready() { return Ok(None); }
        Ok(Some(PinchManager {
            fingers: FingerPair::new(primary,secondary),
            best_pinch: PixelPinchAction { scale: 1., eigenpoint: 0., delta_y: 0. },
            initial_screen: ScreenPosition::new(stage)?,
            min_sep, min_scale
        }))
    }

    pub(crate) fn set_position(&mut self, primary: (f64,f64), secondary: (f64,f64)) {
        self.fingers.set_position(primary,secondary);
        if let Some(best_pinch) = self.fingers.make_pinch_action(self.min_sep,self.min_scale) {
            self.best_pinch = best_pinch;
        }
    }

    pub(crate) fn pinch(&self) -> &PixelPinchAction { &self.best_pinch }
    pub(crate) fn position(&self) -> ScreenPosition { ScreenPosition::transform(&self.initial_screen,&self.best_pinch) }
}

pub(crate) struct PinchManagerFactory {
    min_sep: f64,
    min_scale: f64
}

impl PinchManagerFactory {
    pub(crate) fn new(config: &PointerConfig) -> PinchManagerFactory {
        PinchManagerFactory {
            min_sep: config.pinch_min_sep,
            min_scale: config.pinch_min_scale
        }
    }

    pub(crate) fn create(&self, stage: &ReadStage, primary: (f64,f64), secondary: (f64,f64)) -> Result<Option<PinchManager>,Message> {
        PinchManager::new(stage,primary,secondary,self.min_sep,self.min_scale)
    }
}
