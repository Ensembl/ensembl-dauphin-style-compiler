pub struct AxisPhysicsConfig {
    pub lethargy: f64,
    pub boing: f64,
    pub vel_min: f64,
    pub force_min: f64,
    pub brake_mul: f64
}

pub(super) struct AxisPhysics {
    config: AxisPhysicsConfig,
    target: Option<f64>,
    brake: bool,
    velocity: f64,
    target_speed: Option<f64>,
}

impl AxisPhysics {
    pub(super) fn new(config: AxisPhysicsConfig) -> AxisPhysics {
        AxisPhysics {
            config,
            target_speed: None,
            brake: false,
            target: None,
            velocity: 0.
        }
    }

    pub(super) fn halt(&mut self) {
        self.target = None;
        self.brake = false;
    }

    pub(super) fn move_to(&mut self, position: f64) {
        self.target = Some(position);
    }

    pub(super) fn move_more(&mut self, amount: f64) {
        if let Some(target) = &mut self.target {
            *target += amount;
        }
        self.velocity = 0.;
    }

    pub(super) fn pull(&mut self, speed: f64, start: bool) {
        self.target_speed = if start { Some(speed) } else { None };
        if !start { self.brake = true; }
    }

    pub(super) fn apply_spring(&mut self, mut current: f64, mut total_dt: f64) -> f64 {
        use web_sys::console;
        if let Some(target) = self.target {
            let crit = (4./self.config.lethargy).sqrt()/self.config.boing; /* critically damped when BOING = 1.0 */
            while total_dt > 0. {
                let dt = total_dt.min(1.);
                total_dt -= dt;
                let drive = target - current;
                let mut drive_f = drive/self.config.lethargy;
                let mut friction_f =  self.velocity * crit;
                if self.brake {
                    friction_f *= self.config.brake_mul;
                    drive_f = 0.;
                }
                let force = drive_f-friction_f;
                self.velocity += force * dt;
                let delta = self.velocity*dt;
                current += delta;
                if self.velocity.abs() < self.config.vel_min && force.abs() < self.config.force_min {
                    self.halt();
                    break;
                }
            }
            current
        } else {
            current
        }
    }

    pub(super) fn tick(&mut self, dt: f64) -> Option<f64> {
        self.target_speed.map(|speed| speed*dt)
    }

    pub(super) fn have_target(&self) -> bool { self.target.is_some() }

    pub(super) fn active(&self) -> bool {
        self.target_speed.is_some() || self.have_target()
    }
}
