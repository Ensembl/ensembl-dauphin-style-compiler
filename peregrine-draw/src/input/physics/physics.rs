use std::sync::{ Arc, Mutex };
use commander::cdr_tick;

use crate::{ PeregrineAPI };
use crate::run::{ PgPeregrineConfig,  PgConfigKey };
use crate::input::{InputEvent, InputEventKind };
use crate::input::low::lowlevel::LowLevelInput;
use crate::util::Message;
use crate::PgCommanderWeb;

#[derive(Clone,Debug,PartialEq,Eq)]
pub enum PullState {
    None,
    Left,
    Right
}

pub struct PhysicsState {
    pull_state: PullState,
    pull_speed: f64,
    pull_max_speed: f64,
    pull_accelleration: f64,
    pull_to: Option<f64>
}

impl PhysicsState {
    fn new(config: &PgPeregrineConfig) -> Result<PhysicsState,Message> {
        Ok(PhysicsState {
            pull_state: PullState::None,
            pull_speed: 0.,
            pull_max_speed: config.get_f64(&PgConfigKey::PullMaxSpeed)?,
            pull_accelleration: config.get_f64(&PgConfigKey::PullAccelleration)?,
            pull_to: None
        })
    }

    fn update_pull(&mut self, api: &PeregrineAPI) {
        /* update pull speed */
        let direction = match self.pull_state {
            PullState::Left => -1.,
            PullState::Right => 1.,
            PullState::None => { self.pull_speed = 0.; self.pull_to = None; return; }
        };
        self.pull_speed += direction * self.pull_accelleration;
        if self.pull_speed > self.pull_max_speed { self.pull_speed = self.pull_max_speed; }
        if self.pull_speed < -self.pull_max_speed { self.pull_speed = -self.pull_max_speed; }
        /* do the pulling */
        if self.pull_to.is_none() { self.pull_to = api.x(); }
        if let (Some(pull_to),Some(bp_per_screen)) = (&mut self.pull_to,api.bp_per_screen()) { 
            *pull_to += self.pull_speed * bp_per_screen;
            api.set_x(*pull_to);
        }
    }

    fn physics_step(&mut self, api: &PeregrineAPI) -> Result<(),Message> {
        self.update_pull(api);
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
        let new_direction = match event.details {
            InputEventKind::PullLeft => PullState::Left,
            InputEventKind::PullRight => PullState::Right,
            _ => { return; }
        };
        let mut state = self.state.lock().unwrap();
        if event.start && state.pull_state == PullState::None {
            state.pull_state = new_direction;
        } else if !event.start && state.pull_state == new_direction {
            state.pull_state = PullState::None;
        }
        use web_sys::console;
        console::log_1(&format!("event: {:?}",state.pull_state).into());
    }

    fn incoming_event(&self, event: &InputEvent) {
        self.incoming_pull_event(event);
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
        low_level.distributor_mut().add(move |e| out2.incoming_event(e));
        let out2 = out.clone();
        let api2 = api.clone();
        commander.add("physics", 0, None, None, Box::pin(async move { out2.physics_loop(&api2).await }));
        Ok(out)
    }
}
