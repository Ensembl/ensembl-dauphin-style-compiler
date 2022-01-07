use crate::input::translate::{axisphysics::{AxisPhysics, AxisPhysicsConfig}, measure::Measure};
use super::regime::{TickResult, RegimeCreator, RegimeTrait};

pub(super) struct DragRegimeCreator(pub AxisPhysicsConfig, pub AxisPhysicsConfig);

impl RegimeCreator for DragRegimeCreator {
    type Object = DragRegime;

    fn create(&self) -> Self::Object {
        DragRegime::new(&self.0,&self.1)
    }
}

pub(crate) struct DragRegime {
    x: AxisPhysics,
    z: AxisPhysics,
    zoom_centre: Option<f64>,
    size: Option<f64>
}

impl DragRegime {
    pub(crate) fn new(x_config: &AxisPhysicsConfig, z_config: &AxisPhysicsConfig) -> DragRegime {
        let x =  AxisPhysics::new(x_config);
        let mut z =  AxisPhysics::new(z_config);
        z.set_min_value(z_config.min_bp_per_screen);
        DragRegime { x, z, zoom_centre: None, size: None }
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

impl RegimeTrait for DragRegime {
    fn set_size(&mut self, measure: &Measure, size: Option<f64>) {
        if let Some(size) = size {
            self.size = Some(size);
            self.z.set_max_value(size);
        }
        self.update_settings(measure);
    }
    
    fn update_settings(&mut self, measure: &Measure) {
        let target_bp_per_screen = self.z.get_target().unwrap_or(measure.bp_per_screen);
        let px_per_bp = measure.px_per_screen / target_bp_per_screen;
        self.x.set_factor(px_per_bp);
        self.x.set_min_value(target_bp_per_screen/2.);
        if let Some(size) = &self.size {
            self.x.set_max_value(*size  - target_bp_per_screen/2.);
        }
        self.x.enforce_limits(measure.x_bp);
    }

    fn tick(&mut self, measure: &Measure, total_dt: f64) -> TickResult {
        if !self.x.is_active() && !self.z.is_active() { return TickResult::Finished; }
        let mut new_x = self.x.apply_spring(measure.x_bp,total_dt);
        let mut new_bp = None;
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
        TickResult::Update(new_x,new_bp,false)
    }
}
