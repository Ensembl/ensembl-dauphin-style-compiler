use super::{animqueue::ApplyResult, axisphysics::{AxisPhysics, AxisPhysicsConfig, Scaling}, measure::Measure};

pub(super) struct PhysicsRunnerWRegime {
    w_left: AxisPhysics,
    w_right: AxisPhysics,
    w_scale: f64,
    size: Option<f64>
}

impl PhysicsRunnerWRegime {
    pub(super) fn new(measure: &Measure, size: Option<f64>) -> PhysicsRunnerWRegime {
        let lethargy = 500.;  // 2500 for keys & animate, 500 for mouse, 50000 for goto
        let w_config = AxisPhysicsConfig {
            lethargy,
            boing: 1.,
            vel_min: 0.0005,
            force_min: 0.00001,
            brake_mul: 0.2,
            scaling: Scaling::Linear(1.)
        };
        let mut w_left = AxisPhysics::new(w_config.clone());
        w_left.set_min_value(0.);
        let mut out = PhysicsRunnerWRegime {
            w_left,
            w_right: AxisPhysics::new(w_config),
            w_scale: 1., 
            size           
        };
        out.update_settings(measure);
        out
    }

    pub(super) fn update_settings(&mut self, measure: &Measure) {
        let px_per_bp = measure.px_per_screen / measure.bp_per_screen;
        if let Some(size) = &self.size {
            self.w_right.set_max_value(size*px_per_bp);
        }
    }

    pub(super) fn set(&mut self, measure: &Measure, centre: f64, scale: f64) {
        let new_left_bp = centre - (scale/2.);
        let new_right_bp = centre + (scale/2.);
        self.w_scale = measure.bp_per_screen / measure.px_per_screen; // bp_per_px
        let min_right_for_zscale = (new_left_bp + 30.)/self.w_scale;
        let right = (new_right_bp/self.w_scale).max(min_right_for_zscale);
        self.w_left.move_to(new_left_bp/self.w_scale);
        self.w_right.move_to(right);
    }

    pub(super) fn apply_spring(&mut self, measure: &Measure, total_dt: f64) -> ApplyResult {
        if !self.w_left.is_active() && !self.w_right.is_active() { return ApplyResult::Finished; }
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
            return ApplyResult::Update(Some(x),Some(bp));
        }
        ApplyResult::Update(None,None)
    }

    pub(super) fn report_target(&self, measure: &Measure) -> (Option<f64>,Option<f64>) {
        let px_per_bp = measure.px_per_screen / measure.bp_per_screen;
        let left = self.w_left.get_target().map(|x| x / px_per_bp);
        let right = self.w_right.get_target().map(|x| x / px_per_bp);
        if let (Some(left),Some(right)) = (left,right) {
            (Some((left+right)/2.),Some(right-left))
        } else {
            (None,None)
        }
    }
}
