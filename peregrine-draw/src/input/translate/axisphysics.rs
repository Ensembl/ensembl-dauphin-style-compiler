#[derive(Clone)]
pub enum Scaling {
    Linear(f64),
    Logarithmic(f64)
}

impl Scaling {
    fn to_internal(&self, factor: f64, value: f64) -> f64 {
        match self {
            Scaling::Linear(k) => value * k * factor,
            Scaling::Logarithmic(k) => value.log2() * k * factor
        }
    }

    fn to_external(&self, factor: f64, value: f64) -> f64 {
        match self {
            Scaling::Linear(k) => value / k / factor,
            Scaling::Logarithmic(k) => 2_f64.powf(value/k/factor)
        }
    }
}

pub(super) struct Puller {
    target_speed: Option<f64>,
}

impl Puller {
    pub(super) fn new() -> Puller {
        Puller {
            target_speed: None
        }
    }

    pub(super) fn pull(&mut self, speed: Option<f64>) {
        self.target_speed = speed;
    }

    pub(super) fn tick(&mut self, dt: f64) -> Option<f64> {
        self.target_speed.map(|speed| speed*dt)
    }

    pub(super) fn is_active(&self) -> bool {
        self.target_speed.is_some()
    }
}

#[derive(Clone)]
pub struct AxisPhysicsConfig {
    pub lethargy: f64,
    pub boing: f64,
    pub vel_min: f64,
    pub force_min: f64,
    pub brake_mul: f64,
    pub scaling: Scaling
}

pub(super) struct AxisPhysics {
    config: AxisPhysicsConfig,
    target: Option<f64>,
    immediate: bool,
    brake: bool,
    velocity: f64,
    min_value: Option<f64>,
    max_value: Option<f64>,
    factor: f64
}

impl AxisPhysics {
    pub(super) fn new(config: AxisPhysicsConfig) -> AxisPhysics {
        AxisPhysics {
            config,
            brake: false,
            target: None,
            immediate: false,
            velocity: 0.,
            min_value: None,
            max_value: None,
            factor: 1.
        }
    }

    pub(super) fn set_factor(&mut self, factor: f64) {
        self.factor = factor;
    }

    pub(super) fn set_min_value(&mut self, min_value: f64) {
        self.min_value = Some(self.config.scaling.to_internal(self.factor,min_value));
        self.apply_limits();
    }

    pub(super) fn set_max_value(&mut self, max_value: f64) {
        self.max_value = Some(self.config.scaling.to_internal(self.factor,max_value));
        self.apply_limits();
    }

    fn apply_limits(&mut self) {
        if let (Some(target),Some(min_value)) = (&mut self.target,self.min_value) {
            if *target < min_value {
                *target = min_value;
            }
        }
        if let (Some(target),Some(max_value)) = (&mut self.target,self.max_value) {
            if *target > max_value {
                *target = max_value;
            }
        }
    }

    pub(super) fn brake(&mut self) { self.brake = true; }

    fn halt(&mut self) {
        self.target = None;
        self.brake = false;
    }

    pub(super) fn set(&mut self, position: f64) {
        self.move_to(position);
        self.immediate = true;
    }

    pub(super) fn move_to(&mut self, position: f64) {
        self.target = Some(self.config.scaling.to_internal(self.factor,position));
        self.apply_limits();
    }

    pub(super) fn move_more(&mut self, amount: f64) {
        if let Some(target) = &mut self.target {
            *target += amount;
        }
        self.apply_limits();
        self.velocity = 0.;
    }

    pub(super) fn apply_spring(&mut self, current: f64, mut total_dt: f64) -> Option<f64> {
        let mut current_px = self.config.scaling.to_internal(self.factor,current);
        if let Some(target) = self.target {
            if self.immediate {
                self.immediate = false;
                self.halt();
                Some(self.config.scaling.to_external(self.factor,target))
            } else {
                let crit = (4./self.config.lethargy).sqrt()/self.config.boing; /* critically damped when BOING = 1.0 */
                while total_dt > 0. {
                    let dt = total_dt.min(0.1);
                    total_dt -= dt;
                    let drive = target - current_px;
                    let mut drive_f = drive/self.config.lethargy;
                    let mut friction_f =  self.velocity * crit;
                    if self.brake {
                        friction_f *= self.config.brake_mul;
                        drive_f = 0.;
                    }
                    let force = drive_f-friction_f;
                    self.velocity += force * dt;
                    let delta = self.velocity*dt;
                    current_px += delta;
                    if self.velocity.abs() < self.config.vel_min && force.abs() < self.config.force_min {
                        current_px = target;
                        self.halt();
                        break;
                    }
                }
                Some(self.config.scaling.to_external(self.factor,current_px))
            }
        } else {
            None
        }
    }

    pub(super) fn is_active(&self) -> bool { self.target.is_some() }

    pub(super) fn get_target(&self) -> Option<f64> {
        self.target.map(|pip| self.config.scaling.to_external(self.factor,pip))
    }
}
