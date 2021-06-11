use std::sync::{ Arc, Mutex };
use commander::cdr_tick;
use crate::{PeregrineAPI, util::needed::{Needed, NeededLock}};
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
    physics_lock: Option<NeededLock>
}

impl PullSpeed {
    fn new(max_speed: f64, acceleration: f64) ->PullSpeed {
        PullSpeed {
            current_dec: false,
            current_inc: false,
            speed: 0.,
            max_speed,
            acceleration,
            physics_lock: None
        }
    }

    fn set_direction(&mut self, physics_needed: &Needed, direction: PullDirection, start: bool) {
        *match direction {
            PullDirection::Decrease => &mut self.current_dec,
            PullDirection::Increase => &mut self.current_inc
        }=start;
        self.physics_lock = Some(physics_needed.lock());
    }

    fn step(&mut self) -> bool {
        /* update pull speed */
        let direction = match (self.current_dec,self.current_inc) {
            (true,false) => -1.,
            (false,true) => 1.,
            _ => { 
                self.speed = 0.;
                self.physics_lock.take();
                return false;
            }
        };
        if self.speed * direction < 0. { self.speed = 0.; } // direction change, so halt immediately
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
    pull_z_to: Option<f64>,
    zoom_px_speed: f64,
}

impl PhysicsState {
    fn new(config: &PgPeregrineConfig) -> Result<PhysicsState,Message> {
        Ok(PhysicsState {
            pull_x_speed: PullSpeed::new(config.get_f64(&PgConfigKey::PullMaxSpeed)?,config.get_f64(&PgConfigKey::PullAcceleration)?),
            pull_x_to: None,
            pull_z_speed: PullSpeed::new(config.get_f64(&PgConfigKey::ZoomMaxSpeed)?,config.get_f64(&PgConfigKey::ZoomAcceleration)?),
            pull_z_to: None,
            zoom_px_speed: config.get_f64(&PgConfigKey::ZoomPixelSpeed)?
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

    fn zoom(&mut self, api: &PeregrineAPI, amount_px: f64, position: Option<(f64,f64)>) -> Result<(),Message> {
        let px_per_screen = api.size().map(|x| x.0 as f64);
        let bp_per_screen = api.bp_per_screen()?;
        let x = api.x()?;
        if let (Some(x),Some(px_per_screen),Some(bp_per_screen)) = (x,px_per_screen,bp_per_screen) {
            let x_screen = if let Some(position) = position { position.0/px_per_screen } else { 0.5 };
            let x_bp = x + (x_screen - 0.5) * bp_per_screen;
            let factor = 2_f64.powf(amount_px/self.zoom_px_speed);
            let new_bp_per_screen = bp_per_screen*factor;
            let new_bp_from_middle = (x_screen-0.5)*new_bp_per_screen;
            let new_middle = x_bp - new_bp_from_middle;
            api.set_bp_per_screen(new_bp_per_screen);
            api.set_x(new_middle);
        }
        Ok(())
    }

    fn scale(&mut self, api: &PeregrineAPI, scale: f64, centre: f64, y: f64) {
        api.set_bp_per_screen(scale);
        api.set_x(centre);
        api.set_y(y);
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
    state: Arc<Mutex<PhysicsState>>,
    physics_needed: Needed
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
        target.set_direction(&self.physics_needed,new_direction,event.start);
    }

    fn incoming_jump_request(&self, api: &PeregrineAPI, event: &InputEvent) -> Result<(),Message> {
        if !event.start { return Ok(()); }
        let mut state = self.state.lock().unwrap();
        let distance = *event.amount.get(0).unwrap_or(&0.);
        let pos_x = event.amount.get(1);
        let pos_y = event.amount.get(2);
        let pos = if let (Some(x),Some(y)) = (pos_x,pos_y) { Some((*x,*y)) } else { None };
        match event.details {
            InputEventKind::PixelsLeft => { state.jump(api,-distance)?; },
            InputEventKind::PixelsRight => { state.jump(api,distance)?; },
            InputEventKind::PixelsIn => { state.zoom(api,-distance,pos)?; },
            InputEventKind::PixelsOut => { state.zoom(api,distance,pos)?; },
            _ => {}
        }
        Ok(())
    }

    fn incoming_scale_event(&self, api: &PeregrineAPI, event: &InputEvent) -> Result<(),Message> {
        if !event.start { return Ok(()); }
        let mut state = self.state.lock().unwrap();
        let scale = *event.amount.get(0).unwrap_or(&1.);
        let centre = *event.amount.get(1).unwrap_or(&0.);
        let y = *event.amount.get(2).unwrap_or(&0.);
        match event.details {
            InputEventKind::SetPosition => {
                state.scale(api,scale,centre,0.);
            },
            _ => {}
        }
        Ok(())
    }

    fn incoming_event(&self, api: &PeregrineAPI, event: &InputEvent) -> Result<(),Message> {
        self.incoming_pull_event(event);
        self.incoming_jump_request(api,event)?;
        self.incoming_scale_event(api,event)?;
        Ok(())
    }

    async fn physics_loop(&self, api: &PeregrineAPI) -> Result<(),Message> {
        loop {
            self.state.lock().unwrap().physics_step(api)?;
            cdr_tick(1).await;
            self.physics_needed.wait_until_needed().await;
        }
    }

    pub fn new(config: &PgPeregrineConfig, low_level: &mut LowLevelInput, api: &PeregrineAPI, commander: &PgCommanderWeb) -> Result<Physics,Message> {
        let out = Physics {
            state: Arc::new(Mutex::new(PhysicsState::new(config)?)),
            physics_needed: Needed::new()
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
