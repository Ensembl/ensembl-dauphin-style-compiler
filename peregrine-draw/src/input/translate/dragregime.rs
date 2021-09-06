use super::{animqueue::{ApplyResult, PhysicsRegimeCreator, PhysicsRegimeTrait}, axisphysics::{AxisPhysics, AxisPhysicsConfig, Scaling}, measure::Measure};

pub(super) struct PhysicsDragRegimeCreator(pub AxisPhysicsConfig, pub AxisPhysicsConfig);

impl PhysicsRegimeCreator for PhysicsDragRegimeCreator {
    type Object = PhysicsRunnerDragRegime;

    fn create(&self) -> Self::Object {
        PhysicsRunnerDragRegime::new(&self.0,&self.1)
    }
}

pub(super) struct PhysicsRunnerDragRegime {
    x: AxisPhysics,
    z: AxisPhysics,
    zoom_centre: Option<f64>,
    size: Option<f64>
}

impl PhysicsRunnerDragRegime {
    pub(crate) fn new(x_config: &AxisPhysicsConfig, z_config: &AxisPhysicsConfig) -> PhysicsRunnerDragRegime {
        let x =  AxisPhysics::new(x_config);
        let mut z =  AxisPhysics::new(z_config);
        z.set_min_value(z_config.min_bp_per_screen);
        PhysicsRunnerDragRegime { x, z, zoom_centre: None, size: None }
    }

    pub(crate) fn shift_to(&mut self, x: f64) {
        self.x.move_to(x);
    }

    pub(crate) fn zoom_to(&mut self, amount: f64) {
        self.z.move_to(amount);
    }

    pub(crate) fn shift_more(&mut self, measure: &Measure, amount_px: f64) {
        if !self.x.is_active() {
            self.x.move_to(measure.x_bp);
        }
        self.x.move_more(amount_px);
    }

    pub(crate) fn zoom_more(&mut self, measure: &Measure, amount_px: f64, centre: Option<f64>) {
        if !self.z.is_active() {
            self.zoom_centre = centre.clone();
            self.z.move_to(measure.bp_per_screen);
        }
        self.z.move_more(amount_px);
    }

    pub(crate) fn brake_x(&mut self) { self.x.brake(); }
    pub(crate) fn brake_z(&mut self) { self.z.brake(); }
}

impl PhysicsRegimeTrait for PhysicsRunnerDragRegime {
    fn set_size(&mut self, measure: &Measure, size: Option<f64>) {
        if let Some(size) = size {
            self.size = Some(size);
            self.z.set_max_value(size);
        }
        self.update_settings(measure);
    }
    
    fn report_target(&mut self, measure: &Measure) -> (Option<f64>,Option<f64>) {
        let px_per_bp = measure.px_per_screen / measure.bp_per_screen;
        self.x.set_factor(px_per_bp);
        let x_bp = self.x.get_target();
        let z_bp = self.z.get_target();
        (x_bp,z_bp)
    }

    fn update_settings(&mut self, measure: &Measure) {
        let target_bp_per_screen = self.z.get_target().unwrap_or(measure.bp_per_screen);
        let px_per_bp = measure.px_per_screen / target_bp_per_screen;
        self.x.set_factor(px_per_bp);
        self.x.set_min_value(target_bp_per_screen/2.);
        if let Some(size) = &self.size {
            self.x.set_max_value(*size  - target_bp_per_screen/2.);
        }
        if measure.x_bp < target_bp_per_screen/2. {
            self.x.set(target_bp_per_screen/2.);
        }
    }

    fn apply_spring(&mut self, measure: &Measure, total_dt: f64) -> ApplyResult {
        if !self.x.is_active() && !self.z.is_active() { return ApplyResult::Finished; }
        let mut new_x = self.x.apply_spring(measure.x_bp,total_dt);
        let mut new_bp = None;
        /* x-coordinate */
        if let Some(new_bp_per_screen) = self.z.apply_spring(measure.bp_per_screen,total_dt) {
            if let Some(stationary) = self.zoom_centre {
                let x_screen = stationary/measure.px_per_screen;
                let new_bp_from_middle = (x_screen-0.5)*new_bp_per_screen;
                let x_bp = measure.x_bp + (x_screen - 0.5) * measure.bp_per_screen;
                let new_middle = x_bp - new_bp_from_middle;
                /* TODO use limits */
                if let Some(size) = self.size {
                    let max_new_middle = size - new_bp_per_screen/2. ;                        
                    if new_x.is_none() { new_x = Some(new_middle.min(max_new_middle)); }
                }
            }
            new_bp = Some(new_bp_per_screen);
        }
        /**/
        self.update_settings(measure);
        ApplyResult::Update(new_x,new_bp)
    }
}