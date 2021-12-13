use crate::{input::translate::{axisphysics::{AxisPhysics, AxisPhysicsConfig}, measure::Measure}};

use super::regime::{TickResult, RegimeCreator, RegimeTrait};

pub(super) struct ZoomXRegimeCreator(pub AxisPhysicsConfig);

impl RegimeCreator for ZoomXRegimeCreator {
    type Object = ZoomXRegime;

    fn create(&self) -> Self::Object {
        ZoomXRegime::new(&self.0)
    }
}

pub(crate) struct ZoomXRegime {
    zoom_x: AxisPhysics,
    size: Option<f64>,
    min_bp: f64
}

impl ZoomXRegime {
    pub(super) fn new(config: &AxisPhysicsConfig) -> ZoomXRegime {
        let mut config = config.clone();
        config.vel_min *= 1000.;
        config.force_min *= 1000.;
        let mut zoom_x = AxisPhysics::new(&config);
        zoom_x.set_factor(1./100.);
        zoom_x.set_min_value(0.);
        ZoomXRegime {
            zoom_x,
            size: None,
            min_bp: config.min_bp_per_screen
        }
    }

    pub(crate) fn set(&mut self, _measure: &Measure, centre: f64) {
        self.zoom_x.move_to(centre);
    }

    fn fixed_bp(&self, pos: f64, bp: f64) -> Option<f64> {
        if let Some(size) = self.size {
            let at_right = pos + bp/2. > size;
            let at_left = pos < bp/2.;
            match (at_left,at_right) {
                (false,false) => Some(bp),
                (true,false) => Some(pos*2.),
                (false,true) => Some((size-pos)*2.),
                (true,true) => None
            }
        } else {
            Some(bp)
        }
    }
}

impl RegimeTrait for ZoomXRegime {
    fn set_size(&mut self, measure: &Measure, size: Option<f64>) {
        if let Some(size) = size {
            self.size = Some(size);
        }
        self.update_settings(measure);
    }

    fn update_settings(&mut self, measure: &Measure) {
        if let Some(size) = &self.size {
            self.zoom_x.set_max_value(size - self.min_bp/2.);
        }
        self.zoom_x.enforce_limits(measure.x_bp);
    }

    fn tick(&mut self, measure: &Measure, total_dt: f64) -> TickResult {
        if !self.zoom_x.is_active() { return TickResult::Finished; }
        let new_pos = self.zoom_x.apply_spring(measure.x_bp,total_dt);
        if let Some(new_pos) = new_pos {
            /* increase bp-per-screen to accommodate it */
            if let Some(new_bp) = self.fixed_bp(new_pos,measure.bp_per_screen) {
                return TickResult::Update(Some(new_pos),Some(new_bp));
            } else {
                return TickResult::Finished;
            }
        }
        TickResult::Update(None,None)
    }
}
