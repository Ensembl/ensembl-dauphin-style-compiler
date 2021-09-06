use std::{sync::{ Arc, Mutex }};
use commander::cdr_tick;
use js_sys::Date;
use peregrine_toolkit::sync::{blocker::{Blocker, Lockout}, needed::{Needed, NeededLock}};
use crate::{ PeregrineInnerAPI, input::translate::measure::Measure, run::report::Report };
use crate::run::{ PgPeregrineConfig };
use crate::input::{InputEvent, InputEventKind };
use crate::input::low::lowlevel::LowLevelInput;
use crate::util::Message;
use crate::PgCommanderWeb;
use super::{animqueue::{Cadence, QueueEntry}, axisphysics::{Puller}};
use super::animqueue::PhysicsRunner;

const PULL_SPEED : f64 = 2.; // px/ms

pub struct PhysicsState {
    runner: PhysicsRunner,
    x_puller: Puller,
    z_puller: Puller,
    last_update: Option<f64>,
    /* used internally to stop spin-waits */
    physics_needed: Needed,
    physics_lock: Option<NeededLock>,
    /* used to implement sync in draw queue */
    queue_blocker: Blocker,
    #[allow(unused)]
    queue_lockout: Option<Lockout>
}

impl PhysicsState {
    fn new(config: &PgPeregrineConfig, physics_needed: &Needed, queue_blocker: &Blocker) -> Result<PhysicsState,Message> {
        Ok(PhysicsState {
            runner: PhysicsRunner::new(config)?,
            last_update: None,
            x_puller: Puller::new(),
            z_puller: Puller::new(),
            physics_needed: physics_needed.clone(),
            physics_lock: None,
            queue_blocker: queue_blocker.clone(),
            queue_lockout: None
        })
    }

    fn pull_x(&mut self, speed: f64, start: bool) -> Result<(),Message> {
        if start {
            self.x_puller.pull(Some(speed));
        } else {
            self.x_puller.pull(None);
            self.runner.queue_add(QueueEntry::BrakeX);
        }
        self.update_needed();
        Ok(())
    }

    fn pull_z(&mut self, speed: f64, start: bool) -> Result<(),Message> {
        if start {
            self.z_puller.pull(Some(speed));
        } else {
            self.z_puller.pull(None);
            self.runner.queue_add(QueueEntry::BrakeZ);
        }
        self.update_needed();
        Ok(())
    }

    fn scale(&mut self, scale: f64, centre_bp: f64, y: f64) -> Result<(),Message> {
        self.runner.queue_clear();
        self.runner.queue_add(QueueEntry::MoveW(centre_bp,scale));
        self.update_needed();
        Ok(())
    }

    fn apply_ongoing(&mut self, dt: f64) -> Result<(),Message> {
        if let Some(delta) = self.x_puller.tick(dt) {
            self.runner.queue_add(QueueEntry::ShiftMore(delta));
            self.update_needed();
        }
        if let Some(delta) = self.z_puller.tick(dt) {
            self.runner.queue_add(QueueEntry::ZoomMore(delta,None));
            self.update_needed();
        }
        Ok(())
    }

    fn update_needed(&mut self) {
        if self.runner.update_needed() || self.x_puller.is_active() || self.z_puller.is_active() {
            if self.physics_lock.is_none() {
                self.physics_lock = Some(self.physics_needed.lock());
                self.queue_lockout = Some(self.queue_blocker.lock());
            }
        } else {
            self.physics_lock = None;
            self.queue_lockout = None;
            self.last_update = None;
        }
    }

    fn physics_step(&mut self, inner: &mut PeregrineInnerAPI, report: &mut Report) -> Result<(),Message> {
        let now = Date::now();
        if let Some(last_update) = self.last_update {
            let dt = now - last_update;
            self.apply_ongoing(dt)?;
            self.runner.drain_animation_queue(inner,report)?;
            self.runner.apply_spring(inner,dt)?;
        }
        self.last_update = Some(now);
        self.update_needed();
        Ok(())
    }

    fn goto_not_ready(&mut self, inner: &mut PeregrineInnerAPI, centre: f64, bp_per_screen: f64) -> Result<(),Message> {
        inner.set_x(centre);
        inner.set_bp_per_screen(bp_per_screen);
        Ok(())
    }

    fn animate_to(&mut self, inner: &mut PeregrineInnerAPI, centre: f64, bp_per_screen: f64, cadence: &Cadence) -> Result<(),Message> {
        let measure = if let Some(measure) = Measure::new(inner)? { measure } else { return Ok(()); };
        self.runner.queue_clear();
        /* three strategies:
         * 1. target is smaller: make short move and then zoom in
         * 2. target is bigger: zoom out to common scale move and make short move
         * 3. outzoom to target scale shift and soom in again
         */
         if bp_per_screen > measure.bp_per_screen {
            /* we are getting more bp per screen, ie zooming out: can use strategies 2 or 3 */
            /* to test if we can use 2: how many screenfuls must we move at the FINAL scale? */
            let screenful_move = (centre-measure.x_bp).abs() / bp_per_screen;
            if screenful_move < 2. { // XXX config
                /* strategy 2 */
                self.runner.queue_add(QueueEntry::ZoomTo(bp_per_screen,cadence.clone()));
                self.runner.queue_add(QueueEntry::Wait);
                self.runner.queue_add(QueueEntry::ShiftTo(centre,cadence.clone()));
                self.update_needed();
                return Ok(());
            }
        } else {
            /* we are getting fewer bp per screen, ie zooming in: can use strategies 1 or 3 */
            /* to test if we can use 1: how many screenfuls must we move at the ORIGINAL scale? */
            let screenful_move = (centre-measure.x_bp).abs() / measure.bp_per_screen;
            if screenful_move < 2. { // XXX config
                /* strategy 1 */
                self.runner.queue_add(QueueEntry::ShiftTo(centre,cadence.clone()));
                self.runner.queue_add(QueueEntry::Wait);
                self.runner.queue_add(QueueEntry::ShiftByZoomTo(centre,cadence.clone()));
                self.runner.queue_add(QueueEntry::Wait);
                self.runner.queue_add(QueueEntry::ZoomTo(bp_per_screen,cadence.clone()));
                self.update_needed();
                return Ok(());
            }
        }
        /* strategy 3 */
        let rightmost = (centre+bp_per_screen/2.).max(measure.x_bp+measure.bp_per_screen/2.);
        let leftmost = (centre-bp_per_screen/2.).min(measure.x_bp-measure.bp_per_screen/2.);
        let outzoom_bp_per_screen = (rightmost-leftmost)*2.;
        self.runner.queue_add(QueueEntry::ZoomTo(outzoom_bp_per_screen,cadence.clone()));
        self.runner.queue_add(QueueEntry::Wait);
        self.runner.queue_add(QueueEntry::ShiftTo(centre,cadence.clone()));
        self.runner.queue_add(QueueEntry::Wait);
        self.runner.queue_add(QueueEntry::ShiftByZoomTo(centre,cadence.clone()));
        self.runner.queue_add(QueueEntry::Wait);
        self.runner.queue_add(QueueEntry::ZoomTo(bp_per_screen,cadence.clone()));
        self.update_needed();
        Ok(())
    }

    fn goto(&mut self, inner: &mut PeregrineInnerAPI, centre: f64, bp_per_screen: f64) -> Result<(),Message> {
        let ready = inner.stage().lock().unwrap().ready();
        if ready {
            self.animate_to(inner,centre,bp_per_screen,&Cadence::SelfPropelled)?;
        } else {
            self.goto_not_ready(inner,centre,bp_per_screen)?;
        }
        Ok(())
    }

    pub fn set_limit(&mut self, limit: f64) {
        self.runner.queue_add(QueueEntry::Size(limit));
    }
}

#[derive(Clone)]
pub struct Physics {
    state: Arc<Mutex<PhysicsState>>,
    report: Report,
    physics_needed: Needed
}

// XXX blur halt

impl Physics {
    fn incoming_pull_event(&self, event: &InputEvent) -> Result<(),Message> {
        let mut state = self.state.lock().unwrap();
        match event.details {
            InputEventKind::PullLeft => state.pull_x(-PULL_SPEED*2.,event.start)?,
            InputEventKind::PullRight => state.pull_x(PULL_SPEED*2.,event.start)?,
            InputEventKind::PullIn => state.pull_z(-PULL_SPEED,event.start)?,
            InputEventKind::PullOut => state.pull_z(PULL_SPEED,event.start)?,
            _ => {}
        }
        Ok(())
    }

    fn incoming_jump_request(&self, event: &InputEvent) -> Result<(),Message> {
        if !event.start { return Ok(()); }
        let mut state = self.state.lock().unwrap();
        let distance = *event.amount.get(0).unwrap_or(&0.);
        let pos_x = event.amount.get(1);
        let pos_y = event.amount.get(2);
        let mut centre = None;
        if let Some(x) = pos_x {
            centre = Some(*x);
        }
        match event.details {
            InputEventKind::PixelsLeft => {
                state.runner.queue_add(QueueEntry::ShiftMore(-distance));
                state.update_needed();
            },
            InputEventKind::PixelsRight => {
                state.runner.queue_add(QueueEntry::ShiftMore(distance));
                state.update_needed();
            },
            InputEventKind::PixelsIn => {
                state.runner.queue_add(QueueEntry::ZoomMore(-distance,centre));               
                state.update_needed();
            },
            InputEventKind::PixelsOut => {
                state.runner.queue_add(QueueEntry::ZoomMore(distance,centre));
                state.update_needed();
            },
            _ => {}
        }
        Ok(())
    }

    fn incoming_scale_event(&self, event: &InputEvent) -> Result<(),Message> {
        if !event.start { return Ok(()); }
        let mut state = self.state.lock().unwrap();
        let scale = *event.amount.get(0).unwrap_or(&1.);
        let centre = *event.amount.get(1).unwrap_or(&0.);
        let y = *event.amount.get(2).unwrap_or(&0.);
        match event.details {
            InputEventKind::SetPosition => {
                self.report.set_target_x_bp(centre);
                self.report.set_target_bp_per_screen(scale);        
                state.scale(scale,centre,0.)?;
            },
            _ => {}
        }
        Ok(())
    }

    fn incoming_animate_event(&self, inner: &mut PeregrineInnerAPI, event: &InputEvent) -> Result<(),Message> {
        if !event.start { return Ok(()); }
        let mut state = self.state.lock().unwrap();
        let scale = *event.amount.get(0).unwrap_or(&1.);
        let centre = *event.amount.get(1).unwrap_or(&0.);
        let y = *event.amount.get(2).unwrap_or(&0.);
        match event.details {
            InputEventKind::AnimatePosition => {
                // XXX y
                self.report.set_target_x_bp(centre);
                self.report.set_target_bp_per_screen(scale);        
                state.animate_to(inner,centre,scale,&Cadence::Instructed)?;
            },
            _ => {}
        }
        Ok(())
    }

    fn incoming_event(&self, inner: &mut PeregrineInnerAPI, event: &InputEvent) -> Result<(),Message> {
        self.incoming_pull_event(event)?;
        self.incoming_jump_request(event)?;
        self.incoming_scale_event(event)?;
        self.incoming_animate_event(inner,event)?;
        Ok(())
    }

    async fn physics_loop(&self, inner: &mut PeregrineInnerAPI) -> Result<(),Message> {
        let lweb = inner.lock().await;
        let mut report = lweb.report.clone();
        drop(lweb);
        loop {
            self.state.lock().unwrap().physics_step(inner,&mut report)?;
            cdr_tick(1).await;
            self.physics_needed.wait_until_needed().await;
        }
    }

    pub fn new(config: &PgPeregrineConfig, low_level: &mut LowLevelInput, inner: &PeregrineInnerAPI, commander: &PgCommanderWeb, report: &Report, queue_blocker: &Blocker) -> Result<Physics,Message> {
        let physics_needed = Needed::new();
        let out = Physics {
            state: Arc::new(Mutex::new(PhysicsState::new(config,&physics_needed,queue_blocker)?)),
            report: report.clone(),
            physics_needed: physics_needed.clone()
        };
        let out2 = out.clone();
        let mut inner2 = inner.clone();
        low_level.distributor_mut().add(move |e| { out2.incoming_event(&mut inner2,e).ok(); }); // XXX error distribution
        let out2 = out.clone();
        let mut inner2 = inner.clone();
        commander.add("physics", 0, None, None, Box::pin(async move { out2.physics_loop(&mut inner2).await }));
        Ok(out)
    }

    pub fn goto(&self, api: &mut PeregrineInnerAPI, centre: f64, scale: f64) -> Result<(),Message> {
        self.report.set_target_x_bp(centre);
        self.report.set_target_bp_per_screen(scale);
        self.state.lock().unwrap().goto(api,centre,scale)
    }

    pub fn set_limit(&self, limit: f64) {
        self.state.lock().unwrap().set_limit(limit);
    }
}
