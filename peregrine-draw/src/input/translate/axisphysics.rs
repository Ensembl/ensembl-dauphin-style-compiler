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
    pub scaling: Scaling,
    pub min_bp_per_screen: f64
}

pub(super) enum Stopped {
    Nominal,
    Minimum,
    Maximum
}

pub(super) struct AxisPhysics {
    config: AxisPhysicsConfig,
    stopped: Stopped,
    target: Option<f64>,
    immediate: bool,
    brake: bool,
    velocity: f64,
    min_value: Option<f64>,
    max_value: Option<f64>,
    factor: f64
}

impl AxisPhysics {
    pub(super) fn new(config: &AxisPhysicsConfig) -> AxisPhysics {
        AxisPhysics {
            config: config.clone(),
            stopped: Stopped::Nominal,
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
        self.apply_limits();
    }

    pub(super) fn set_min_value(&mut self, min_value: f64) {
        self.min_value = Some(min_value);
        self.apply_limits();
    }

    pub(super) fn set_max_value(&mut self, max_value: f64) {
        self.max_value = Some(max_value);
        self.apply_limits();
    }

    pub(super) fn limited_value(&self, mut value: f64) -> (f64,Stopped) {
        let mut stopped = Stopped::Nominal;
        if let Some(min_value) = self.min_value {
            if value < min_value {
                value = min_value;
                stopped = Stopped::Minimum;
            }
        }
        if let Some(max_value) = self.max_value {
            if value > max_value {
                value = max_value;
                stopped = Stopped::Maximum;
            }
        }
        (value,stopped)
    }

    fn apply_limits(&mut self) {        
        let mut limited = None;
        if let Some(target) = self.target {
            limited = Some(self.limited_value(target));
        }
        if let Some((value,stopped)) = limited {
            self.target = Some(value);
            self.stopped = stopped;
        }
    }

    pub(super) fn is_stopped(&self) -> &Stopped { &self.stopped }
    pub(super) fn brake(&mut self) { self.brake = true; }

    fn halt(&mut self) {
        self.target = None;
        self.brake = false;
    }

    pub(super) fn enforce_limits(&mut self, position: f64) {
        let limited_position = self.limited_value(position).0;
        if position != limited_position {
            self.target = Some(limited_position);
            self.immediate = true;    
        }
    }

    pub(super) fn move_to(&mut self, position: f64) {
        self.target = Some(position);
        self.apply_limits();
    }

    pub(super) fn move_more(&mut self, amount: f64) {
        if let Some(target) = &mut self.target {
            let mut target_px = self.config.scaling.to_internal(self.factor,*target);
            target_px += amount;
            *target = self.config.scaling.to_external(self.factor,target_px);
        }
        self.apply_limits();
    }

    pub(super) fn apply_spring(&mut self, current: f64, mut total_dt: f64) -> Option<f64> {
        let mut current_px = self.config.scaling.to_internal(self.factor,current);
        if let Some(target) = self.target {
            if self.immediate {
                self.immediate = false;
                self.halt();
                Some(target)
            } else {
                let target_px = self.config.scaling.to_internal(self.factor,target);
                let crit = (4./self.config.lethargy).sqrt()/self.config.boing; /* critically damped when BOING = 1.0 */
                while total_dt > 0. {
                    let dt = total_dt.min(0.1);
                    total_dt -= dt;
                    let drive = target_px - current_px;
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
                        current_px = target_px;
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

    pub(super) fn get_target(&self) -> Option<f64> { self.target }
}
