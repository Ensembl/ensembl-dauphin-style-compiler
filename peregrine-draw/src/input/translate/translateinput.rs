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

    fn goto_not_ready(&mut self, inner: &mut PeregrineInnerAPI, centre: f64, bp_per_screen: f64) -> Result<(),Message> {
        inner.set_x(centre);
        inner.set_bp_per_screen(bp_per_screen);
        self.target_reporter.force_report();
        Ok(())
    }

    fn animate_to(&mut self, inner: &mut PeregrineInnerAPI, centre: f64, bp_per_screen: f64, cadence: &Cadence) -> Result<(),Message> {
        let measure = if let Some(measure) = Measure::new(inner)? { measure } else { return Ok(()); };
        self.queue.queue_clear();
        /* three strategies:
         * 1. target is smaller: make short move, and then zoom in
         * 2. target is bigger: zoom out to common scale, and make short move
         * 3. outzoom to target scale, shift, and zoom in again
         */
         if bp_per_screen > measure.bp_per_screen {
            /* we are getting more bp per screen, ie zooming out: can use strategies 2 or 3 */
            /* to test if we can use 2: how many screenfuls must we move at the FINAL scale? */
            let screenful_move = (centre-measure.x_bp).abs() / bp_per_screen;
            if screenful_move < 2. { // XXX config
                /* strategy 2 */
                self.queue.queue_add(QueueEntry::LockReports);
                self.queue.queue_add(QueueEntry::ZoomTo(bp_per_screen,cadence.clone()));
                self.queue.queue_add(QueueEntry::Wait);
                self.queue.queue_add(QueueEntry::ShiftTo(centre,cadence.clone()));
                self.queue.queue_add(QueueEntry::Wait);
                self.queue.queue_add(QueueEntry::Report);
                self.update_needed();
                return Ok(());
            }
        } else {
            /* we are getting fewer bp per screen, ie zooming in: can use strategies 1 or 3 */
            /* to test if we can use 1: how many screenfuls must we move at the ORIGINAL scale? */
            let screenful_move = (centre-measure.x_bp).abs() / measure.bp_per_screen;
            if screenful_move < 2. { // XXX config
                /* strategy 1 */
                self.queue.queue_add(QueueEntry::LockReports);
                self.queue.queue_add(QueueEntry::ShiftTo(centre,cadence.clone()));
                self.queue.queue_add(QueueEntry::Wait);
                self.queue.queue_add(QueueEntry::ShiftByZoomTo(centre,cadence.clone()));
                self.queue.queue_add(QueueEntry::Wait);
                self.queue.queue_add(QueueEntry::ZoomTo(bp_per_screen,cadence.clone()));
                self.queue.queue_add(QueueEntry::Wait);
                self.queue.queue_add(QueueEntry::Report);
                self.update_needed();
                return Ok(());
            }
        }
        /* strategy 3 */
        let rightmost = (centre+bp_per_screen/2.).max(measure.x_bp+measure.bp_per_screen/2.);
        let leftmost = (centre-bp_per_screen/2.).min(measure.x_bp-measure.bp_per_screen/2.);
        let outzoom_bp_per_screen = (rightmost-leftmost)*2.;
        self.queue.queue_add(QueueEntry::LockReports);
        self.queue.queue_add(QueueEntry::ZoomTo(outzoom_bp_per_screen,cadence.clone()));
        self.queue.queue_add(QueueEntry::Wait);
        self.queue.queue_add(QueueEntry::ShiftTo(centre,cadence.clone()));
        self.queue.queue_add(QueueEntry::Wait);
        self.queue.queue_add(QueueEntry::ShiftByZoomTo(centre,cadence.clone()));
        self.queue.queue_add(QueueEntry::Wait);
        self.queue.queue_add(QueueEntry::ZoomTo(bp_per_screen,cadence.clone()));
        self.queue.queue_add(QueueEntry::Wait);
        self.queue.queue_add(QueueEntry::Report);
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

    fn set_limit(&mut self, limit: f64) {
        self.queue.queue_add(QueueEntry::Size(limit));
    }
}

#[derive(Clone)]
pub struct InputTranslator {
    state: Arc<Mutex<InputTranslatorState>>,
    report: Report,
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
                state.animate_to(inner,centre,scale,&Cadence::Instructed)?;
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
        let mut report = lweb.report.clone();
        drop(lweb);
        loop {
            self.state.lock().unwrap().physics_step(inner,&mut report)?;
            cdr_tick(1).await;
            self.physics_needed.wait_until_needed().await;
        }
    }

    pub fn new(config: &PgPeregrineConfig, low_level: &mut LowLevelInput, inner: &PeregrineInnerAPI, commander: &PgCommanderWeb, report: &Report, queue_blocker: &Blocker, target_reporter: &TargetReporter) -> Result<InputTranslator,Message> {
        let physics_needed = Needed::new();
        let out = InputTranslator {
            state: Arc::new(Mutex::new(InputTranslatorState::new(config,&physics_needed,queue_blocker,target_reporter)?)),
            report: report.clone(),
            physics_needed: physics_needed.clone()
        };
        let out2 = out.clone();
        let mut inner2 = inner.clone();
        low_level.distributor_mut().add(move |e| { out2.incoming_event(&mut inner2,e).ok(); }); // XXX error distribution
        let out2 = out.clone();
        let mut inner2 = inner.clone();
        commander.add("translate input", 0, None, None, Box::pin(async move { out2.physics_loop(&mut inner2).await }));
        Ok(out)
    }

    pub fn goto(&self, api: &mut PeregrineInnerAPI, centre: f64, scale: f64) -> Result<(),Message> {
        self.state.lock().unwrap().goto(api,centre,scale)?;
        Ok(())
    }

    pub fn set_limit(&self, limit: f64) {
        self.state.lock().unwrap().set_limit(limit);
    }
}