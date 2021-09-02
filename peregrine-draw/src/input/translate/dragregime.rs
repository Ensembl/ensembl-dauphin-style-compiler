use super::{animqueue::{ApplyResult, bp_to_zpx, zpx_to_bp}, axisphysics::{AxisPhysics, AxisPhysicsConfig}, measure::Measure};

pub(super) struct PhysicsRunnerDragRegime {
    x: AxisPhysics,
    z: AxisPhysics,
    zoom_centre: Option<f64>
}

impl PhysicsRunnerDragRegime {
    pub(crate) fn new() -> PhysicsRunnerDragRegime {
        let lethargy = 500.;  // 2500 for keys & animate, 500 for mouse, 50000 for goto
        let x_config = AxisPhysicsConfig {
            lethargy,
            boing: 1.,
            vel_min: 0.0005,
            force_min: 0.00001,
            brake_mul: 0.2
        };
        let z_config = AxisPhysicsConfig {
            lethargy,
            boing: 1.,
            vel_min: 0.0005,
            force_min: 0.00001,
            brake_mul: 0.2
        };
        let x =  AxisPhysics::new(x_config);
        let mut z =  AxisPhysics::new(z_config);
        z.set_min_value(bp_to_zpx(30.));
        PhysicsRunnerDragRegime { x, z, zoom_centre: None }
    }

    fn update_x_limits(&mut self, measure: &Measure) {
        let target_bp_per_screen = self.z.get_target().map(|z| zpx_to_bp(z)).unwrap_or(measure.bp_per_screen);
        let px_per_bp = measure.px_per_screen / measure.bp_per_screen;
        let x_min_px = target_bp_per_screen*px_per_bp/2.;
        self.x.set_min_value(x_min_px);
        if measure.x_bp * px_per_bp < x_min_px {
            self.x.set(x_min_px);
        }
    }

    pub(crate) fn jump_x(&mut self, measure: &Measure, x: f64) {
        self.x.move_to(x);
        self.update_x_limits(measure);
    }

    pub(crate) fn jump_z(&mut self, measure: &Measure, amount: f64, centre: Option<f64>) {
        self.z.move_to(amount);
        if !self.x.is_active() && centre.is_some() {
            self.x.move_to(centre.unwrap());
        }
        self.update_x_limits(measure);
    }

    pub(crate) fn move_x(&mut self, measure: &Measure, amount_px: f64) {
        let current_px = measure.x_bp / measure.bp_per_screen * measure.px_per_screen;
        if !self.x.is_active() {
            self.x.move_to(current_px);
        }
        self.x.move_more(amount_px);
        self.update_x_limits(measure);
    }

    pub(crate) fn move_z(&mut self, measure: &Measure, amount_px: f64, centre: Option<f64>) {
        let z_current_px = bp_to_zpx(measure.bp_per_screen);
        if !self.z.is_active() {
            self.zoom_centre = centre.clone();
            self.z.move_to(z_current_px);
        }
        self.z.move_more(amount_px);
        self.update_x_limits(measure);
    }

    pub(crate) fn brake_x(&mut self) { self.x.brake(); }
    pub(crate) fn brake_z(&mut self) { self.z.brake(); }

    pub(crate) fn apply_spring(&mut self, measure: &Measure, total_dt: f64) -> ApplyResult {
        if !self.x.is_active() && !self.z.is_active() { return ApplyResult::Finished; }
        let mut new_x = None;
        let mut new_bp = None;
        /* x-coordinate */
        let px_per_bp = measure.px_per_screen / measure.bp_per_screen;
        if let Some(new_pos_px) = self.x.apply_spring(measure.x_bp*px_per_bp,total_dt) {
            new_x = Some(new_pos_px / px_per_bp);
        }
        /* z-coordinate */
        let z_current_px = bp_to_zpx(measure.bp_per_screen);
        if let Some(new_pos_px) = self.z.apply_spring(z_current_px,total_dt) {
            let new_bp_per_screen = zpx_to_bp(new_pos_px);
            if let Some(stationary) = self.zoom_centre {
                let x_screen = stationary/measure.px_per_screen;
                let new_bp_from_middle = (x_screen-0.5)*new_bp_per_screen;
                let x_bp = measure.x_bp + (x_screen - 0.5) * measure.bp_per_screen;
                let new_middle = x_bp - new_bp_from_middle;
                if new_x.is_none() { new_x = Some(new_middle); }
            }
            new_bp = Some(new_bp_per_screen);
        }
        /**/
        ApplyResult::Update(new_x,new_bp)
    }

    pub(super) fn report_target(&self, measure: &Measure) -> (Option<f64>,Option<f64>) {
        let px_per_bp = measure.px_per_screen / measure.bp_per_screen;
        let x_bp = self.x.get_target().map(|x| x/px_per_bp);
        let z_bp = self.z.get_target().map(|z| zpx_to_bp(z));
        (x_bp,z_bp)
    }
}
