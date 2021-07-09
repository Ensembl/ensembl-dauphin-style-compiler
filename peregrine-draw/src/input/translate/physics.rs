use std::sync::{ Arc, Mutex };
use commander::cdr_tick;
use js_sys::Date;
use crate::{PeregrineAPI, util::needed::{Needed, NeededLock}};
use crate::run::{ PgPeregrineConfig,  PgConfigKey };
use crate::input::{InputEvent, InputEventKind };
use crate::input::low::lowlevel::LowLevelInput;
use crate::util::Message;
use crate::PgCommanderWeb;
use super::axisphysics::AxisPhysics;

const PULL_SPEED : f64 = 2.; // px/ms

pub struct ClosedRamp {
    start: f64,
    end: f64,
    accel: f64,
    max_speed: f64,
    total_distance: f64,
    total_time: f64,
    neg: f64,
}

impl ClosedRamp {
    fn new(accel: f64, speed_limit: f64, start: f64, end: f64) -> ClosedRamp {
        let total_distance = (end - start).abs();
        let neg = if end < start { -1. } else { 1. };
        /* does max_speed need to be reduced due to short distance and so no cruise? */
        let max_speed = (accel*total_distance).sqrt().min(speed_limit);
        let total_time = total_distance/max_speed + max_speed/accel;
        ClosedRamp { accel, max_speed, total_time, total_distance, start, end, neg }    
    }

    fn position_half(&self, time: f64) -> f64 {
        let ramp_time = self.max_speed/self.accel;
        let accel_time = time.min(ramp_time);
        let cruise_time = (time - accel_time).max(0.);
        self.accel * accel_time * accel_time / 2.0 + cruise_time * self.max_speed
    }

    fn position_unit(&self, time: f64) -> f64 {
        if time > self.total_time / 2. {
            self.total_distance - self.position_half(self.total_time - time)
        } else {
            self.position_half(time)
        }
    }

    fn position_linear(&self, time: f64) -> f64 {
        self.start + self.position_unit(time) * self.neg
    }

    fn position_exponential(&self, time: f64) -> f64 {
        self.start*((self.end/self.start).powf(self.position_unit(time)/self.total_distance))
    }

    fn done(&self, time: f64) -> bool { time > self.total_time }
}

pub struct OpenRamp {
    accel: f64,
    speed_limit: f64,
    start: f64,
    negative: bool
}

impl OpenRamp {
    fn new(accel: f64, speed_limit: f64, start: f64, negative: bool) -> OpenRamp {
        OpenRamp { accel, speed_limit, start, negative }
    }

    fn change(&self, time: f64) -> f64 {
        let accel_time = time.min(self.speed_limit/self.accel);
        let cruise_time = (time - self.speed_limit/self.accel).max(0.);
        let change = self.accel*accel_time*accel_time/2.0 + cruise_time * self.speed_limit;
        change
    }

    fn position_linear(&self, time: f64) -> f64 {
        let dir = if self.negative { -1. } else { 1. };
        self.start + self.change(time) * dir
    }

    fn position_exponential(&self, time: f64) -> f64 {
        let dir = if self.negative { -1. } else { 1. };
        self.start * (2_f64).powf(self.change(time) * dir)
    }

    fn negative(&self) -> bool { self.negative }
}

pub struct OpenRampSwitches {
    tick: f64,
    ramp: Option<OpenRamp>,
    speed_limit: f64,
    accel: f64,
}

impl OpenRampSwitches {
    fn new(accel: f64, speed_limit: f64) -> OpenRampSwitches {
        OpenRampSwitches { ramp: None, speed_limit, accel, tick: 0. }
    }

    fn set(&mut self, position: f64, start: bool, negative: bool, scale: f64) {
        if let Some(ramp) = &self.ramp {
            if ramp.negative() == negative { /* same direction */
                if !start {
                    self.ramp = None;
                }
                return;
            } else { /* opposite direction */
                if !start { return; }
            }
        }
        self.ramp = Some(OpenRamp::new(self.accel * scale,self.speed_limit * scale,position,negative));
        self.tick = 0.;
    }

    fn next_point_linear(&mut self) -> Option<f64> {
        let tick = self.tick;
        self.tick += 1.;
        self.ramp.as_mut().map(|ramp| ramp.position_linear(tick))
    }

    fn next_point_exponential(&mut self) -> Option<f64> {
        let tick = self.tick;
        self.tick += 1.;
        self.ramp.as_mut().map(|ramp| ramp.position_exponential(tick))
    }

    fn is_active(&self) -> bool { self.ramp.is_some() }
    fn clear(&mut self) { 
        self.ramp = None;
    }
}

pub struct ClosedRampTimer {
    ramp: ClosedRamp,
    tick: f64
}

impl ClosedRampTimer {
    fn new(accel: f64, speed_limit: f64, start: f64, end: f64, scale: f64) -> ClosedRampTimer {
        ClosedRampTimer {
            ramp: ClosedRamp::new(accel * scale,speed_limit * scale,start,end),
            tick: 0.
        }
    }

    fn next_point_linear(&mut self) -> Option<f64> {
        if self.done() { return None; }
        let tick = self.tick;
        self.tick += 1.;
        Some(self.ramp.position_linear(tick))
    }

    fn next_point_exponential(&mut self) -> Option<f64> {
        if self.done() { return None; }
        let tick = self.tick;
        self.tick += 1.;
        Some(self.ramp.position_exponential(tick))
    }

    fn done(&self) -> bool { self.ramp.done(self.tick) }
}

pub struct PhysicsState {
    x: AxisPhysics,
    last_update: Option<f64>,
    // x_switches: OpenRampSwitches,
    z_switches: OpenRampSwitches,
    // x_target: Option<ClosedRampTimer>,
    z_target: Option<ClosedRampTimer>,
    ready_z_target: Option<ClosedRampTimer>,
    zoom_px_speed: f64,
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
        Ok(PhysicsState {
            last_update: None,
            x: AxisPhysics::new(),
            z_switches: OpenRampSwitches::new(config.get_f64(&PgConfigKey::ZoomAcceleration)?,config.get_f64(&PgConfigKey::ZoomMaxSpeed)?),
            z_target: None,
            ready_z_target: None,
            zoom_px_speed: config.get_f64(&PgConfigKey::ZoomPixelSpeed)?,
            physics_needed: physics_needed.clone(),
            physics_lock: None,
            x_accel,
            x_speed,
            z_accel,
            z_speed
        })
    }

    fn jump(&mut self, api: &PeregrineAPI, amount_px: f64) -> Result<(),Message> {
        if let (Some(current_bp),Some(bp_per_screen),Some(size_px)) = (api.x()?,api.bp_per_screen()?,api.size()) {
            let amount_bp = amount_px / (size_px.0 as f64) * bp_per_screen;
            self.x.jump(current_bp,amount_bp);
        }
        self.update_needed();
        Ok(())
    }

    fn pull_x(&mut self, api: &PeregrineAPI, speed: f64, start: bool) -> Result<(),Message> {
        self.x.pull(speed,start);
        self.update_needed();
        Ok(())
    }

    fn apply_spring(&mut self, api: &PeregrineAPI, total_dt: f64) -> Result<(),Message> {
        let x_current_bp = api.x()?;
        if let (Some(x_current_bp),Some(screen_size),Some(bp_per_screen)) = 
                    (x_current_bp,api.size(),api.bp_per_screen()?) {
            let px_per_screen = screen_size.0 as f64;
            let px_per_bp = px_per_screen / bp_per_screen;
            let new_pos = self.x.apply_spring(px_per_bp, x_current_bp, total_dt);
            api.set_x(new_pos);
        }
        Ok(())
    }

    fn zoom(&mut self, api: &PeregrineAPI, amount_px: f64, position: Option<(f64,f64)>) -> Result<(),Message> {
        // self.x_target = None;
        self.z_target = None;
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
        // self.x_target = None;
        self.z_target = None;
        self.ready_z_target = None;
        api.set_bp_per_screen(scale);
        api.set_x(centre);
        api.set_y(y);
    }

    fn animate_to(&mut self, api: &PeregrineAPI, scale: f64, centre: f64) -> Result<(),Message> {
        // self.x_switches.clear();
        self.z_switches.clear();
        if let (Some(bp_per_screen),Some(x)) = (api.bp_per_screen()?,api.x()?) {
            // self.x_target = Some(ClosedRampTimer::new(self.x_accel,self.x_speed,
            //                 x,centre,bp_per_screen));
            self.ready_z_target = Some(ClosedRampTimer::new(self.z_accel,self.z_speed,
                bp_per_screen,scale,bp_per_screen));    
            self.physics_step(api)?;
        }
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
            use web_sys::console;
            //console::log_1(&format!("delta={}",delta).into());
            self.jump(api,delta)?;
        }
        Ok(())
    }

    fn update_needed(&mut self) {
        if self.x.active() || self.z_switches.is_active() || self.z_target.is_some() {
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
        if let Some(position) = self.z_switches.next_point_exponential() {
            api.set_bp_per_screen(position);
        }
        if /*self.x_target.is_none() &&*/ self.z_target.is_none() {
            self.z_target = self.ready_z_target.take();
        }
        if let Some(position) = self.z_target.as_mut().and_then(|ramp| ramp.next_point_exponential()) {
            api.set_bp_per_screen(position);
        } else {
            self.z_target = None;
        }
        self.update_needed();
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
    fn incoming_pull_event(&self, api: &PeregrineAPI, event: &InputEvent) -> Result<(),Message> {
        let bp_per_screen = match api.bp_per_screen()? { Some(x) => x, None => { return Ok(()); } };
        let mut state = self.state.lock().unwrap();
        match event.details {
            InputEventKind::PullLeft => state.pull_x(api,-PULL_SPEED,event.start)?,
            InputEventKind::PullRight => state.pull_x(api,PULL_SPEED,event.start)?,
            _ => {}
        }

        let (target,position,scale) = match event.details {
            InputEventKind::PullLeft | InputEventKind::PullRight => {
                let position = state.get_x_pull(api)?;
                return Ok(());
                //(&mut state.x_switches,position,bp_per_screen)
            },
            InputEventKind::PullIn | InputEventKind::PullOut => {
                let position = state.get_z_pull(api)?;
                (&mut state.z_switches,position,1.)
            },
            _ => { return Ok(()); }
        };
        let negative = match event.details {
            InputEventKind::PullLeft | InputEventKind::PullIn => { true },
            _ => { false }
        };
        if let Some(position) = position {
            target.set(position,event.start,negative,scale);
            state.physics_step(api)?;
        }
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
