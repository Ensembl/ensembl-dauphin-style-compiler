use std::sync::{ Arc, Mutex };
use commander::cdr_tick;
use crate::{ PeregrineAPI };
use crate::run::{ PgPeregrineConfig,  PgConfigKey };
use crate::input::{InputEvent, InputEventKind };
use crate::input::low::lowlevel::LowLevelInput;
use crate::util::Message;
use crate::PgCommanderWeb;

#[derive(Clone,Debug,PartialEq,Eq)]
pub enum PullDirection {
    Decrease,
    Increase
}

pub struct PullSpeed {
    current_dec: bool,
    current_inc: bool,
    speed: f64,
    max_speed: f64,
    acceleration: f64,
}

impl PullSpeed {
    fn new(max_speed: f64, acceleration: f64) ->PullSpeed {
        PullSpeed {
            current_dec: false,
            current_inc: false,
            speed: 0.,
            max_speed,
            acceleration
        }
    }

    fn set_direction(&mut self, direction: PullDirection, start: bool) {
        *match direction {
            PullDirection::Decrease => &mut self.current_dec,
            PullDirection::Increase => &mut self.current_inc
        }=start;
    }

    fn step(&mut self) -> bool {
        /* update pull speed */
        let direction = match (self.current_dec,self.current_inc) {
            (true,false) => -1.,
            (false,true) => 1.,
            _ => { self.speed = 0.; return false; }
        };
        self.speed += direction * self.acceleration;
        if self.speed > self.max_speed { self.speed = self.max_speed; }
        if self.speed < -self.max_speed { self.speed = -self.max_speed; }
        true
    }

    fn speed(&self) -> f64 { self.speed }
}

pub struct PhysicsState {
    pull_x_speed: PullSpeed,
    pull_x_to: Option<f64>,
    pull_z_speed: PullSpeed,
    pull_z_to: Option<f64>
}

impl PhysicsState {
    fn new(config: &PgPeregrineConfig) -> Result<PhysicsState,Message> {
        Ok(PhysicsState {
            pull_x_speed: PullSpeed::new(config.get_f64(&PgConfigKey::PullMaxSpeed)?,config.get_f64(&PgConfigKey::PullAcceleration)?),
            pull_x_to: None,
            pull_z_speed: PullSpeed::new(config.get_f64(&PgConfigKey::ZoomMaxSpeed)?,config.get_f64(&PgConfigKey::ZoomAcceleration)?),
            pull_z_to: None,
        })
    }

    fn jump(&mut self, api: &PeregrineAPI, amount_px: f64) -> Result<(),Message> {
        self.pull_x_to = None;
        let x = api.x()?;
        let bp_per_screen = api.bp_per_screen()?;
        let px_per_screen = api.size().map(|x| x.0 as f64);
        if let (Some(x),Some(bp_per_screen),Some(px_per_screen)) = (x,bp_per_screen,px_per_screen) {
            let bp_per_px = bp_per_screen/px_per_screen;
            api.set_x(x + amount_px*bp_per_px);
        }
        Ok(())
    }

    fn update_x_pull(&mut self, api: &PeregrineAPI) -> Result<(),Message> {
        if self.pull_x_speed.step() {
            if self.pull_x_to.is_none() { self.pull_x_to = api.x()?; }
            if let (Some(pull_to),Some(bp_per_screen)) = (&mut self.pull_x_to,api.bp_per_screen()?) { 
                *pull_to += self.pull_x_speed.speed() * bp_per_screen;
                api.set_x(*pull_to);
            }
        } else {
            self.pull_x_to = None;
        }
        Ok(())
    }

    fn update_z_pull(&mut self, api: &PeregrineAPI) -> Result<(),Message> {
        if self.pull_z_speed.step() {
            if self.pull_z_to.is_none() { self.pull_z_to = api.bp_per_screen()?; }
            if let Some(pull_to) = &mut self.pull_z_to {
                *pull_to *= (2_f64).powf(self.pull_z_speed.speed());
                api.set_bp_per_screen(*pull_to);
            }
        } else {
            self.pull_z_to = None;
        }
        Ok(())
    }

    fn physics_step(&mut self, api: &PeregrineAPI) -> Result<(),Message> {
        self.update_x_pull(api)?;
        self.update_z_pull(api)?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct Physics {
    state: Arc<Mutex<PhysicsState>>
}

// XXX blur halt

impl Physics {
    fn incoming_pull_event(&self, event: &InputEvent) {
        let mut state = self.state.lock().unwrap();
        let (new_direction,target) = match event.details {
            InputEventKind::PullLeft => (PullDirection::Decrease,&mut state.pull_x_speed),
            InputEventKind::PullRight=> (PullDirection::Increase,&mut state.pull_x_speed),
            InputEventKind::PullIn => (PullDirection::Decrease,&mut state.pull_z_speed),
            InputEventKind::PullOut=> (PullDirection::Increase,&mut state.pull_z_speed),
            _ => { return; }
        };
        target.set_direction(new_direction,event.start);
    }

    fn incoming_jump_request(&self, api: &PeregrineAPI, event: &InputEvent) -> Result<(),Message> {
        if !event.start { return Ok(()); }
        let mut state = self.state.lock().unwrap();
        let distance = *event.amount.get(0).unwrap_or(&0.);
        match event.details {
            InputEventKind::PixelsLeft => { state.jump(api,-distance)?; },
            InputEventKind::PixelsRight => { state.jump(api,distance)?; },
            _ => {}
        }
        Ok(())
    }

    fn incoming_event(&self, api: &PeregrineAPI, event: &InputEvent) -> Result<(),Message> {
        self.incoming_pull_event(event);
        self.incoming_jump_request(api,event)?;
        Ok(())
    }

    async fn physics_loop(&self, api: &PeregrineAPI) -> Result<(),Message> {
        loop {
            self.state.lock().unwrap().physics_step(api)?;
            cdr_tick(1).await;
        }
    }

    pub fn new(config: &PgPeregrineConfig, low_level: &mut LowLevelInput, api: &PeregrineAPI, commander: &PgCommanderWeb) -> Result<Physics,Message> {
        let out = Physics {
            state: Arc::new(Mutex::new(PhysicsState::new(config)?))
        };
        let out2 = out.clone();
        let api2 = api.clone();
        low_level.distributor_mut().add(move |e| { out2.incoming_event(&api2,e).ok(); }); // XXX error distribution
        let out2 = out.clone();
        let api2 = api.clone();
        commander.add("physics", 0, None, None, Box::pin(async move { out2.physics_loop(&api2).await }));
        Ok(out)
    }
}
