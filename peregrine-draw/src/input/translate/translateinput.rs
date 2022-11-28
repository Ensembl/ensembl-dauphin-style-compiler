use std::{sync::{ Arc, Mutex }};
use commander::cdr_tick;
use js_sys::Date;
use peregrine_toolkit::lock;
use peregrine_toolkit_async::sync::{blocker::{Blocker, Lockout}, needed::{Needed, NeededLock}};
use crate::{ PeregrineInnerAPI, run::report::Report };
use crate::run::{ PgPeregrineConfig };
use crate::input::{InputEvent, InputEventKind };
use crate::input::low::lowlevel::LowLevelInput;
use crate::util::Message;
use crate::PgCommanderWeb;
use super::{animqueue::{Cadence, QueueEntry}, axisphysics::{Puller}, targetreporter::TargetReporter};
use super::animqueue::AnimationQueue;

const PULL_SPEED : f64 = 2.; // px/ms

pub struct InputTranslatorState {
    queue: AnimationQueue,
    x_puller: Puller,
    z_puller: Puller,
    last_update: Option<f64>,
    /* used internally to stop spin-waits */
    physics_needed: Needed,
    physics_lock: Option<NeededLock>,
    /* used to implement sync in draw queue */
    queue_blocker: Blocker,
    #[allow(unused)]
    queue_lockout: Option<Lockout>,
    target_reporter: TargetReporter
}

impl InputTranslatorState {
    fn new(config: &PgPeregrineConfig, physics_needed: &Needed, queue_blocker: &Blocker, target_reporter: &TargetReporter) -> Result<InputTranslatorState,Message> {
        Ok(InputTranslatorState {
            queue: AnimationQueue::new(config, target_reporter)?,
            last_update: None,
            x_puller: Puller::new(),
            z_puller: Puller::new(),
            physics_needed: physics_needed.clone(),
            physics_lock: None,
            queue_blocker: queue_blocker.clone(),
            queue_lockout: None,
            target_reporter: target_reporter.clone()
        })
    }

    fn pull_x(&mut self, speed: f64, start: bool) -> Result<(),Message> {
        if start {
            self.x_puller.pull(Some(speed));
        } else {
            self.x_puller.pull(None);
            self.queue.queue_add(QueueEntry::BrakeX);
        }
        self.update_needed();
        Ok(())
    }

    fn pull_z(&mut self, speed: f64, start: bool) -> Result<(),Message> {
        if start {
            self.z_puller.pull(Some(speed));
        } else {
            self.z_puller.pull(None);
            self.queue.queue_add(QueueEntry::BrakeZ);
        }
        self.update_needed();
        Ok(())
    }

    fn apply_ongoing(&mut self, dt: f64) -> Result<(),Message> {
        if let Some(delta) = self.x_puller.tick(dt) {
            self.queue.queue_add(QueueEntry::ShiftMore(delta));
            self.update_needed();
        }
        if let Some(delta) = self.z_puller.tick(dt) {
            self.queue.queue_add(QueueEntry::ZoomMore(delta,None));
            self.update_needed();
        }
        Ok(())
    }

    fn update_needed(&mut self) {
        if self.queue.update_needed() || self.x_puller.is_active() || self.z_puller.is_active() {
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
        let mut finished = false;
        let now = Date::now();
        if let Some(last_update) = self.last_update {
            let dt = now - last_update;
            self.apply_ongoing(dt)?;
            self.queue.drain_animation_queue(inner,report)?;
            finished = self.queue.regime_tick(inner,dt)?;
        }
        self.last_update = Some(now);
        self.update_needed();
        if finished {
            self.target_reporter.apply_force();
        }
        Ok(())
    }

    fn just_goto(&mut self, inner: &mut PeregrineInnerAPI, centre: f64, bp_per_screen: f64, only_if_unknown: bool) -> Result<(),Message> {
        inner.set_position(Some(centre),Some(bp_per_screen),only_if_unknown);
        self.target_reporter.force_report();
        Ok(())
    }

    fn animate_to(&mut self, _inner: &mut PeregrineInnerAPI, centre: f64, bp_per_screen: f64, cadence: &Cadence) -> Result<(),Message> {
        self.queue.remove_pending_actions();
        self.queue.queue_add(QueueEntry::LockReports);
        self.queue.queue_add(QueueEntry::Sketchy(true));
        match cadence {
            Cadence::Smooth => {
                self.queue.queue_add(QueueEntry::Goto(Some(centre),Some(bp_per_screen)));
            },
            Cadence::Step => {
                self.queue.queue_add(QueueEntry::Goto(Some(centre),None));
                self.queue.queue_add(QueueEntry::Wait);
                self.queue.queue_add(QueueEntry::Goto(None,Some(bp_per_screen)));
            }
        }
        self.queue.queue_add(QueueEntry::Wait);
        self.queue.queue_add(QueueEntry::Sketchy(false));
        self.queue.queue_add(QueueEntry::Report);
        self.update_needed();
        return Ok(());
    }

    fn goto(&mut self, inner: &mut PeregrineInnerAPI, centre: f64, bp_per_screen: f64, only_if_unknown: bool) -> Result<(),Message> {
        let ready = lock!(inner.stage()).ready();
        if ready && !only_if_unknown {
            self.animate_to(inner,centre,bp_per_screen,&Cadence::Smooth)?;
        } else {
            self.just_goto(inner,centre,bp_per_screen,only_if_unknown)?;
        }
        Ok(())
    }

    fn set_limit(&mut self, limit: f64) {
        self.queue.queue_add(QueueEntry::Size(limit));
        self.update_needed();
    }
}

#[derive(Clone)]
pub struct InputTranslator {
    state: Arc<Mutex<InputTranslatorState>>,
    physics_needed: Needed
}

// XXX blur halt

impl InputTranslator {
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
        let _pos_y = event.amount.get(2);
        let mut centre = None;
        if let Some(x) = pos_x {
            centre = Some(*x);
        }
        match event.details {
            InputEventKind::PixelsLeft => {
                state.queue.queue_add(QueueEntry::ShiftMore(-distance));
                state.update_needed();
            },
            InputEventKind::PixelsRight => {
                state.queue.queue_add(QueueEntry::ShiftMore(distance));
                state.update_needed();
            },
            InputEventKind::PixelsIn => {
                state.queue.queue_add(QueueEntry::ZoomMore(-distance,centre));               
                state.update_needed();
            },
            InputEventKind::PixelsOut => {
                state.queue.queue_add(QueueEntry::ZoomMore(distance,centre));
                state.update_needed();
            },
            _ => {}
        }
        Ok(())
    }

    fn incoming_set_position(&self, event: &InputEvent) -> Result<(),Message> {
        if !event.start { return Ok(()); }
        let mut state = self.state.lock().unwrap();
        let scale = *event.amount.get(0).unwrap_or(&1.);
        let centre = *event.amount.get(1).unwrap_or(&0.);
        let _y = *event.amount.get(2).unwrap_or(&0.);
        match event.details {
            InputEventKind::SetPosition => {
                state.queue.queue_add(QueueEntry::Set(centre,scale));
                state.update_needed();
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
        let _y = *event.amount.get(2).unwrap_or(&0.);
        match event.details {
            InputEventKind::AnimatePosition => {
                // XXX y
                state.animate_to(inner,centre,scale,&Cadence::Step)?;
            },
            _ => {}
        }
        Ok(())
    }

    fn incoming_event(&self, inner: &mut PeregrineInnerAPI, event: &InputEvent) -> Result<(),Message> {
        self.incoming_pull_event(event)?;
        self.incoming_jump_request(event)?;
        self.incoming_set_position(event)?;
        self.incoming_animate_event(inner,event)?;
        Ok(())
    }

    async fn physics_loop(&self, inner: &mut PeregrineInnerAPI) -> Result<(),Message> {
        let lweb = inner.lock().await;
        let shutdown = lweb.dom.shutdown().clone();
        let needed = self.physics_needed.clone();
        shutdown.add(move || { needed.set(); });
        let mut report = lweb.report.clone();
        drop(lweb);
        while !shutdown.poll() {
            self.state.lock().unwrap().physics_step(inner,&mut report)?;
            cdr_tick(1).await;
            self.physics_needed.wait_until_needed().await;
        }
        Ok(())
    }

    pub fn new(config: &PgPeregrineConfig, low_level: &mut LowLevelInput, inner: &PeregrineInnerAPI, commander: &PgCommanderWeb, queue_blocker: &Blocker, target_reporter: &TargetReporter) -> Result<InputTranslator,Message> {
        let physics_needed = Needed::new();
        let out = InputTranslator {
            state: Arc::new(Mutex::new(InputTranslatorState::new(config,&physics_needed,queue_blocker,target_reporter)?)),
            physics_needed: physics_needed.clone()
        };
        let out2 = out.clone();
        let mut inner2 = inner.clone();
        low_level.distributor_mut().add(move |e| { out2.incoming_event(&mut inner2,e).ok(); }); // XXX error distribution
        let out2 = out.clone();
        let mut inner2 = inner.clone();
        commander.add("translate input", 0, None, None, Box::pin(async move { 
            out2.physics_loop(&mut inner2).await 
        }));
        Ok(out)
    }

    pub fn goto(&self, api: &mut PeregrineInnerAPI, centre: f64, scale: f64, only_if_unknown: bool) -> Result<(),Message> {
        lock!(self.state).goto(api,centre,scale,only_if_unknown)?;
        Ok(())
    }

    pub fn set_limit(&self, limit: f64) {
        lock!(self.state).set_limit(limit);
    }
}
