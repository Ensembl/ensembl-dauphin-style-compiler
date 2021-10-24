use crate::input::translate::{axisphysics::{AxisPhysics, AxisPhysicsConfig}, measure::Measure};

use super::regime::{TickResult, RegimeCreator, RegimeTrait};

pub(super) struct WRegimeCreator(pub AxisPhysicsConfig);

impl RegimeCreator for WRegimeCreator {
    type Object = WRegime;

    fn create(&self) -> Self::Object {
        WRegime::new(&self.0)
    }
}

pub(crate) struct WRegime {
    w_left: AxisPhysics,
    w_right: AxisPhysics,
    w_scale: f64,
    size: Option<f64>,
    min_bp: f64
}

impl WRegime {
    pub(super) fn new(w_config: &AxisPhysicsConfig) -> WRegime {
        let mut w_left = AxisPhysics::new(&w_config);
        w_left.set_min_value(0.);
        WRegime {
            w_left,
            w_right: AxisPhysics::new(&w_config),
            w_scale: 1., 
            size: None,
            min_bp: w_config.min_bp_per_screen
        }
    }

    pub(crate) fn set(&mut self, measure: &Measure, centre: f64, scale: f64) {
        let new_left_bp = centre - (scale/2.);
        let new_right_bp = centre + (scale/2.);
        self.w_scale = measure.bp_per_screen / measure.px_per_screen; // bp_per_px
        let min_right_for_zscale = (new_left_bp + self.min_bp)/self.w_scale;
        let right = (new_right_bp/self.w_scale).max(min_right_for_zscale);
        self.w_left.move_to(new_left_bp/self.w_scale);
        self.w_right.move_to(right);
    }
}

impl RegimeTrait for WRegime {
    fn set_size(&mut self, measure: &Measure, size: Option<f64>) {
        if let Some(size) = size {
            self.size = Some(size);
        }
        self.update_settings(measure);
    }

    fn report_target(&mut self, measure: &Measure) -> (Option<f64>,Option<f64>) {
        let px_per_bp = measure.px_per_screen / measure.bp_per_screen;
        let left = self.w_left.get_target().map(|x| x / px_per_bp);
        let right = self.w_right.get_target().map(|x| x / px_per_bp);
        if let (Some(left),Some(right)) = (left,right) {
            (Some((left+right)/2.),Some(right-left))
        } else {
            (None,None)
        }
    }

    fn update_settings(&mut self, measure: &Measure) {
        let px_per_bp = measure.px_per_screen / measure.bp_per_screen;
        if let Some(size) = &self.size {
            self.w_left.set_max_value(size*px_per_bp);
            self.w_right.set_max_value(size*px_per_bp);
        }
    }

    fn tick(&mut self, measure: &Measure, total_dt: f64) -> TickResult {
        if !self.w_left.is_active() && !self.w_right.is_active() { return TickResult::Finished; }
        /* where are we right now? */
        let old_left_bp = measure.x_bp - measure.bp_per_screen/2.;
        let old_right_bp = measure.x_bp + measure.bp_per_screen/2.;
        let old_left_px = old_left_bp / self.w_scale;
        let old_right_px = old_right_bp / self.w_scale;
        /* how much should we move */
        let new_left_px = self.w_left.apply_spring(old_left_px,total_dt);
        let new_right_px = self.w_right.apply_spring(old_right_px,total_dt);
        let new_pos = match (new_left_px,new_right_px) {
            (Some(left),Some(right)) => Some((left,right)),
            (Some(left),None) => Some((left,old_right_px)),
            (None,Some(right)) => Some((old_left_px,right)),
            (None,None) => None
        };
        if let Some((new_left_px,new_right_px)) = new_pos {
            let new_left_bp = new_left_px * self.w_scale;
            let new_right_bp = new_right_px * self.w_scale;
            /* compute new position */
            let x = (new_left_bp+new_right_bp)/2.;
            let bp = new_right_bp-new_left_bp;
            return TickResult::Update(Some(x),Some(bp));
        }
        TickResult::Update(None,None)
    }
}
