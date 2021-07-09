use std::sync::{ Arc, Mutex };
use commander::cdr_tick;
use js_sys::Date;
use crate::{PeregrineAPI, util::needed::{Needed, NeededLock}};
use crate::run::{ PgPeregrineConfig,  PgConfigKey };
use crate::input::{InputEvent, InputEventKind };
use crate::input::low::lowlevel::LowLevelInput;
use crate::util::Message;
use crate::PgCommanderWeb;
use super::axisphysics::{AxisPhysics, AxisPhysicsConfig};

const PULL_SPEED : f64 = 2.; // px/ms

pub struct PhysicsState {
    x: AxisPhysics,
    z: AxisPhysics,
    last_update: Option<f64>,
    zoom_px_speed: f64,
    zoom_centre: Option<(f64,f64)>,
    z_zoom_centre: Option<(f64,f64)>,
    physics_needed: Needed,
    physics_lock: Option<NeededLock>,
    x_accel: f64,
    x_speed: f64,
    z_accel: f64,
    z_speed: f64,
}

impl PhysicsState {
    fn new(config: &PgPeregrineConfig, physics_needed: &Needed) -> Result<PhysicsState,Message> {
        let x_accel = config.get_f64(&PgConfigKey::PullAcceleration)?;
        let x_speed = config.get_f64(&PgConfigKey::AutomatedPullMaxSpeed)?;
        let z_accel = config.get_f64(&PgConfigKey::ZoomAcceleration)?;
        let z_speed = config.get_f64(&PgConfigKey::AutomatedZoomMaxSpeed)?;
        let x_config = AxisPhysicsConfig {
            lethargy: 100., // 2500 for keys
            boing: 1.,
            vel_min: 0.0005,
            force_min: 0.00001,
            brake_mul: 0.2
        };
        let z_config = AxisPhysicsConfig {
            lethargy: 100.,
            boing: 1.,
            vel_min: 0.0005,
            force_min: 0.00001,
            brake_mul: 0.2
        };
        Ok(PhysicsState {
            last_update: None,
            x: AxisPhysics::new(x_config),
            z: AxisPhysics::new(z_config),
            //z_switches: OpenRampSwitches::new(config.get_f64(&PgConfigKey::ZoomAcceleration)?,config.get_f64(&PgConfigKey::ZoomMaxSpeed)?),
            //z_target: None,
            //ready_z_target: None,
            zoom_px_speed: config.get_f64(&PgConfigKey::ZoomPixelSpeed)?,
            zoom_centre: None,
            z_zoom_centre: None,
            physics_needed: physics_needed.clone(),
            physics_lock: None,
            x_accel,
            x_speed,
            z_accel,
            z_speed
        })
    }

    fn set_zoom_centre(&mut self, pos: (f64,f64)) {
        self.zoom_centre = Some(pos);
    }

    fn jump_x(&mut self, api: &PeregrineAPI, amount_px: f64) -> Result<(),Message> {
        if let (Some(current_bp),Some(bp_per_screen),Some(size_px)) = (api.x()?,api.bp_per_screen()?,api.size()) {
            let current_px = current_bp / bp_per_screen * (size_px.0 as f64);
            if !self.x.have_target() {
                self.x.move_to(current_px);
            }
            self.x.move_more(amount_px);
        }
        self.update_needed();
        Ok(())
    }

    fn jump_z(&mut self, api: &PeregrineAPI, amount_px: f64) -> Result<(),Message> {
        if let Some(bp_per_screen) = api.bp_per_screen()? {
            let z_current_px = bp_to_zpx(bp_per_screen);
            if !self.z.have_target() {
                self.z_zoom_centre = self.zoom_centre;
                self.z.move_to(z_current_px);
            }
            self.z.move_more(amount_px);
        }
        self.update_needed();
        Ok(())
    }

    fn pull_x(&mut self, speed: f64, start: bool) -> Result<(),Message> {
        self.x.pull(speed,start);
        self.update_needed();
        Ok(())
    }

    fn pull_z(&mut self, speed: f64, start: bool) -> Result<(),Message> {
        self.z.pull(speed,start);
        self.update_needed();
        Ok(())
    }

    fn apply_spring_x(&mut self, api: &PeregrineAPI, total_dt: f64) -> Result<(),Message> {
        if !self.x.have_target() { return Ok(()); }
        let x_current_bp = api.x()?;
        if let (Some(x_current_bp),Some(screen_size),Some(bp_per_screen)) = 
                    (x_current_bp,api.size(),api.bp_per_screen()?) {
            let px_per_screen = screen_size.0 as f64;
            let px_per_bp = px_per_screen / bp_per_screen;
            let new_pos_px = self.x.apply_spring(x_current_bp*px_per_bp,total_dt);
            api.set_x(new_pos_px / px_per_bp);
        }
        Ok(())
    }

    fn apply_spring_z(&mut self, api: &PeregrineAPI, total_dt: f64) -> Result<(),Message> {
        if !self.z.have_target() { return Ok(()); }
        let px_per_screen = api.size().map(|x| x.0 as f64);
        let z_current_bp = api.bp_per_screen()?;
        let x = api.x()?;
        if let (Some(x),Some(z_current_bp),Some(screen_size),Some(bp_per_screen)) = 
                    (x,z_current_bp,px_per_screen,api.bp_per_screen()?) {                        
            let z_current_px = bp_to_zpx(z_current_bp);
            let new_pos_px = self.z.apply_spring(z_current_px,total_dt);
            let new_bp_per_screen = zpx_to_bp(new_pos_px);
            api.set_bp_per_screen(new_bp_per_screen);
            if let Some(zoom_centre) = self.z_zoom_centre {
                let x_screen = zoom_centre.0/screen_size;
                let new_bp_from_middle = (x_screen-0.5)*new_bp_per_screen;
                let x_bp = x + (x_screen - 0.5) * bp_per_screen;
                let new_middle = x_bp - new_bp_from_middle;
                api.set_x(new_middle);
            }
        }
        Ok(())
    }

    fn apply_spring(&mut self, api: &PeregrineAPI, total_dt: f64) -> Result<(),Message> {
        self.apply_spring_x(api,total_dt)?;
        self.apply_spring_z(api,total_dt)?;
        Ok(())
    }

    fn scale(&mut self, api: &PeregrineAPI, scale: f64, centre_bp: f64, y: f64) -> Result<(),Message> {
        self.x.halt();
        self.z.halt();
        self.z_zoom_centre = None;
        self.z.move_to( bp_to_zpx(scale));
        api.set_x(centre_bp);
        self.update_needed();
        Ok(())
    }

    fn animate_to(&mut self, api: &PeregrineAPI, scale: f64, centre: f64) -> Result<(),Message> {
        // self.x_switches.clear();
        /*
        self.z_switches.clear();
        if let (Some(bp_per_screen),Some(x)) = (api.bp_per_screen()?,api.x()?) {
            // self.x_target = Some(ClosedRampTimer::new(self.x_accel,self.x_speed,
            //                 x,centre,bp_per_screen));
            self.ready_z_target = Some(ClosedRampTimer::new(self.z_accel,self.z_speed,
                bp_per_screen,scale,bp_per_screen));    
            self.physics_step(api)?;
        }
        */
        Ok(())
    }

    fn get_x_pull(&mut self, api: &PeregrineAPI) -> Result<Option<f64>,Message> {
        api.x()
    }

    fn get_z_pull(&mut self, api: &PeregrineAPI) -> Result<Option<f64>,Message> {
        api.bp_per_screen()
    }

    fn apply_ongoing(&mut self, api: &PeregrineAPI, dt: f64) -> Result<(),Message> {
        if let Some(delta) = self.x.tick(dt) {
            self.jump_x(api,delta)?;
        }
        if let Some(delta) = self.z.tick(dt) {
            self.jump_z(api,delta)?;
        }
        Ok(())
    }

    fn update_needed(&mut self) {
        if self.x.active() || self.z.active() {
            if self.physics_lock.is_none() {
                self.physics_lock = Some(self.physics_needed.lock());
            }
        } else {
            self.physics_lock = None;
            self.last_update = None;
        }
    }

    fn physics_step(&mut self, api: &PeregrineAPI) -> Result<(),Message> {
        let now = Date::now();
        if let Some(last_update) = self.last_update {
            let dt = now - last_update;
            self.apply_ongoing(api,dt)?;
            self.apply_spring(api,dt)?;
        }
        self.last_update = Some(now);
        self.update_needed();
        Ok(())
    }
}

fn bp_to_zpx(bp: f64) -> f64 { bp.log2() * 100. }
fn zpx_to_bp(zpx: f64) -> f64 { 2_f64.powf(zpx/100.) }

#[derive(Clone)]
pub struct Physics {
    state: Arc<Mutex<PhysicsState>>,
    physics_needed: Needed
}

// XXX blur halt

impl Physics {
    fn incoming_pull_event(&self, api: &PeregrineAPI, event: &InputEvent) -> Result<(),Message> {
        let bp_per_screen = match api.bp_per_screen()? { Some(x) => x, None => { return Ok(()); } };
        let mut state = self.state.lock().unwrap();
        match event.details {
            InputEventKind::PullLeft => state.pull_x(-PULL_SPEED*2.,event.start)?,
            InputEventKind::PullRight => state.pull_x(PULL_SPEED*2.,event.start)?,
            InputEventKind::PullIn => state.pull_z(-PULL_SPEED,event.start)?,
            InputEventKind::PullOut => state.pull_z(PULL_SPEED,event.start)?,
            _ => {}
        }

        /*
        let (target,position,scale) = match event.details {
            InputEventKind::PullLeft | InputEventKind::PullRight => {
                let position = state.get_x_pull(api)?;
                return Ok(());
                //(&mut state.x_switches,position,bp_per_screen)
            },
            InputEventKind::PullIn | InputEventKind::PullOut => {
                let position = state.get_z_pull(api)?;
                return Ok(());
            },
            _ => { return Ok(()); }
        };
        */
        /*
        let negative = match event.details {
            InputEventKind::PullLeft | InputEventKind::PullIn => { true },
            _ => { false }
        };
        if let Some(position) = position {
            target.set(position,event.start,negative,scale);
            state.physics_step(api)?;
        }
        */
        Ok(())
    }

    fn incoming_animate_event(&self, api: &PeregrineAPI, event: &InputEvent) -> Result<(),Message> {
        if !event.start { return Ok(()); }
        let mut state = self.state.lock().unwrap();
        let scale = *event.amount.get(0).unwrap_or(&1.);
        let centre = *event.amount.get(1).unwrap_or(&0.);
        let y = *event.amount.get(2).unwrap_or(&0.);
        match event.details {
            InputEventKind::AnimatePosition => {
                // XXX y
                state.animate_to(api,scale,centre)?;
            },
            _ => {}
        }
        Ok(())
    }

    fn incoming_jump_request(&self, api: &PeregrineAPI, event: &InputEvent) -> Result<(),Message> {
        if !event.start { return Ok(()); }
        let mut state = self.state.lock().unwrap();
        let distance = *event.amount.get(0).unwrap_or(&0.);
        let pos_x = event.amount.get(1);
        let pos_y = event.amount.get(2);
        if let (Some(x),Some(y)) = (pos_x,pos_y) {
            state.set_zoom_centre((*x,*y));
        }
        match event.details {
            InputEventKind::PixelsLeft => { state.jump_x(api,-distance)?; },
            InputEventKind::PixelsRight => { state.jump_x(api,distance)?; },
            InputEventKind::PixelsIn => { state.jump_z(api,-distance)?; },
            InputEventKind::PixelsOut => { state.jump_z(api,distance)?; },
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
               state.scale(api,scale,centre,0.)?;
            },
            _ => {}
        }
        Ok(())
    }

    fn incoming_event(&self, api: &PeregrineAPI, event: &InputEvent) -> Result<(),Message> {
        self.incoming_pull_event(api,event)?;
        self.incoming_jump_request(api,event)?;
        self.incoming_scale_event(api,event)?;
        self.incoming_animate_event(api,event)?;
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
        let physics_needed = Needed::new();
        let out = Physics {
            state: Arc::new(Mutex::new(PhysicsState::new(config,&physics_needed)?)),
            physics_needed: physics_needed.clone()
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
