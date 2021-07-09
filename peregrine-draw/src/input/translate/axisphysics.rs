use crate::Message;

const LETHARGY : f64 = 2500.;
const BOING : f64 = 1.;
const VEL_MIN : f64 = 0.0005; // px/ms
const FORCE_MIN : f64 = 0.000001; // px/ms/ms
const BRAKE_MUL : f64 = 0.2;

pub(super) struct AxisPhysics {
    target: Option<f64>,
    brake: bool,
    velocity: f64,
    target_speed: Option<f64>,
}

impl AxisPhysics {
    pub(super) fn new() -> AxisPhysics {
        AxisPhysics {
            target_speed: None,
            brake: false,
            target: None,
            velocity: 0.
        }
    }

    pub(super) fn jump(&mut self, current_bp: f64, amount_bp: f64) {
        let target = &mut self.target;
        if target.is_none() { *target = Some(current_bp); }
        if let Some(target) = target {
            *target += amount_bp;
        }
        self.velocity = 0.;
    }

    pub(super) fn pull(&mut self, speed: f64, start: bool) {
        self.target_speed = if start { Some(speed) } else { None };
        if !start { self.brake = true; }
    }

    pub(super) fn apply_spring(&mut self, px_per_bp: f64, mut current_bp: f64, mut total_dt: f64) -> f64 {
        if let Some(target_bp) = self.target {
            let crit = (4./LETHARGY).sqrt()/BOING; /* critically damped when BOING = 1.0 */
            let mut stop = false;
            while total_dt > 0. {
                let dt = total_dt.min(1.);
                total_dt -= dt;
                let drive_bp = target_bp - current_bp;
                let drive_px = drive_bp * px_per_bp;
                let mut drive_f = drive_px/LETHARGY;
                let mut friction_f =  self.velocity * crit;
                if self.brake { friction_f *= BRAKE_MUL; drive_f = 0.; }
                let force = drive_f-friction_f;
                self.velocity += force * dt;
                let delta_x_px = self.velocity*dt;
                let delta_bp = delta_x_px / px_per_bp;
                current_bp += delta_bp;
                if self.velocity.abs() < VEL_MIN && force.abs() < FORCE_MIN { stop = true; }
            }
            if stop {
                self.target = None;
                self.brake = false;
            }
            current_bp
        } else {
            current_bp
        }
    }

    pub(super) fn tick(&mut self, dt: f64) -> Option<f64> {
        self.target_speed.map(|speed| speed*dt)
    }

    pub(super) fn active(&self) -> bool {
        self.target_speed.is_some() || self.target.is_some()
    }
}
