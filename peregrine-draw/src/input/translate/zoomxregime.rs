use crate::{input::translate::axisphysics::Stopped, util::message::Endstop};

use super::{animqueue::{ApplyResult, PhysicsRegimeCreator, PhysicsRegimeTrait}, axisphysics::{AxisPhysics, AxisPhysicsConfig, Scaling}, measure::Measure};

pub(super) struct PhysicsZoomXRegimeCreator(pub AxisPhysicsConfig);

impl PhysicsRegimeCreator for PhysicsZoomXRegimeCreator {
    type Object = PhysicsRunnerZoomXRegime;

    fn create(&self) -> Self::Object {
        PhysicsRunnerZoomXRegime::new(&self.0)
    }
}

pub(super) struct PhysicsRunnerZoomXRegime {
    zoom_x: AxisPhysics,
    size: Option<f64>,
    min_bp: f64
}

impl PhysicsRunnerZoomXRegime {
    pub(super) fn new(config: &AxisPhysicsConfig) -> PhysicsRunnerZoomXRegime {
        let mut config = config.clone();
        config.vel_min *= 1000.;
        config.force_min *= 1000.;
        let mut zoom_x = AxisPhysics::new(&config);
        zoom_x.set_factor(1./100.);
        zoom_x.set_min_value(0.);
        PhysicsRunnerZoomXRegime {
            zoom_x,
            size: None,
            min_bp: config.min_bp_per_screen
        }
    }

    pub(super) fn set(&mut self, measure: &Measure, centre: f64) {
        self.zoom_x.move_to(centre);
    }

    fn fixed_bp(&self, pos: f64, bp: f64) -> f64 {
        let mut new_bp = bp;
        if let Some(size) = self.size {
            if pos + new_bp/2. > size {
                new_bp = (size-pos)*2.;
            } else if pos < new_bp/2. {
                new_bp = pos*2.;
            }
        }
        new_bp
    }
}

impl PhysicsRegimeTrait for PhysicsRunnerZoomXRegime {
    fn set_size(&mut self, measure: &Measure, size: Option<f64>) {
        if let Some(size) = size {
            self.size = Some(size);
        }
        self.update_settings(measure);
    }

    fn report_target(&mut self, measure: &Measure) -> (Option<f64>,Option<f64>) {
        let px_per_bp = measure.px_per_screen / measure.bp_per_screen;
        if let Some(x) = self.zoom_x.get_target() {
            let bp = self.fixed_bp(x,measure.bp_per_screen);
            (Some(x),Some(bp))
        } else {
            (None,None)
        }
    }

    fn update_settings(&mut self, measure: &Measure) {
        if let Some(size) = &self.size {
            self.zoom_x.set_max_value(size - self.min_bp/2.);
        }
        self.zoom_x.enforce_limits(measure.x_bp);
    }

    fn apply_spring(&mut self, measure: &Measure, total_dt: f64) -> ApplyResult {
        if !self.zoom_x.is_active() { return ApplyResult::Finished; }
        let new_pos = self.zoom_x.apply_spring(measure.x_bp,total_dt);
        if let Some(new_pos) = new_pos {
            /* increase bp-per-screen to accommodate it */
            let new_bp = self.fixed_bp(new_pos,measure.bp_per_screen);
            return ApplyResult::Update(Some(new_pos),Some(new_bp));
        }
        ApplyResult::Update(None,None)
    }
}
