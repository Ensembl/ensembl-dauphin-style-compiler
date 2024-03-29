use std::sync::{Arc, Mutex};
use peregrine_data::DataMessage;
use peregrine_toolkit_async::sync::{blocker::{Blocker, Lockout}, needed::Needed};
use peregrine_toolkit::{lock, plumbing::oneshot::OneShot};
use crate::{Message, PgCommanderWeb, run::{PgConfigKey, PgPeregrineConfig, report::Report}, util::debounce::Debounce};

/* Lockable, debounced intention reoprting */

 #[derive(Clone)]
struct TargetLocation {
    stick: Option<String>,
    x: Option<f64>,
    bp_per_screen: Option<f64>
}

impl TargetLocation {
    fn empty() -> TargetLocation {
        TargetLocation {
            stick: None,
            x: None,
            bp_per_screen: None
        }
    }

    fn is_ready(&self) -> bool {
        self.stick.is_some() && self.x.is_some() && self.bp_per_screen.is_some()
    }

    fn make_report(&self, report: &Report) -> bool {
        if self.is_ready() {
            report.set_target_stick(&self.stick.as_ref().unwrap());
            report.set_target_x_bp(self.x.unwrap());
            report.set_target_bp_per_screen(self.bp_per_screen.unwrap());
            return true;
        }
        false
    }
}

struct TargetReporterState {
    report: Report,
    in_stage: Arc<Mutex<TargetLocation>>,  // latest report
    out_stage: Arc<Mutex<TargetLocation>>, // ready to send
    commander: PgCommanderWeb,
    force_needed: Needed,
    needed: Needed,         // pending update
    blocker: Blocker,       // block reports
    debounce: Debounce  // debounce
}

#[derive(Clone)]
pub struct TargetReporter(Arc<Mutex<TargetReporterState>>);

impl TargetReporter {
    async fn report_loop(&self, shutdown: OneShot) -> Result<(),DataMessage> {
        let needed = lock!(self.0).needed.clone();
        let needed2 = needed.clone();
        let blocker = lock!(self.0).blocker.clone();
        shutdown.add(move || { 
            needed2.set();
            blocker.set_freewheel(true);
        });
        while !shutdown.poll() {
            needed.wait_until_needed().await;
            let state = lock!(self.0);
            let blocker = state.blocker.clone();
            drop(state);
            blocker.wait().await;
            let state = lock!(self.0);
            let value = lock!(state.in_stage).clone();
            if value.is_ready() {
                // XXX non-critical race condition but examine all uses of Blocker
                *lock!(state.out_stage) = value;
                state.debounce.set();
            }
            drop(state);
        }
        Ok(())
    }

    pub(crate) fn new(commander: &PgCommanderWeb, shutdown: &OneShot, config: &PgPeregrineConfig, report: &Report) -> Result<TargetReporter,Message> {
        let out_stage :Arc<Mutex<TargetLocation>> = Arc::new(Mutex::new(TargetLocation::empty()));
        let out_stage2 = out_stage.clone();
        let report2 = report.clone();
        let debounce = Debounce::new(commander,config.get_f64(&PgConfigKey::TargetReportTime)?, &shutdown.clone(),move || { // XXX configurable
            lock!(out_stage2).make_report(&report2);
        });
        let out = TargetReporter(Arc::new(Mutex::new(TargetReporterState {
            report: report.clone(),
            out_stage,
            in_stage: Arc::new(Mutex::new(TargetLocation::empty())),
            force_needed: Needed::new(),
            needed: Needed::new(),
            blocker: Blocker::new(),
            commander: commander.clone(),
            debounce,
        })));
        let out2 = out.clone();
        let shutdown = shutdown.clone();
        commander.add("target-reporter", 0, None, None, Box::pin(async move { 
            out2.report_loop(shutdown).await
        }));
        Ok(out)
    }

    pub(crate) fn lock_updates(&self) -> Lockout {
        lock!(self.0).blocker.lock()
    }

    async fn force_applier(&self) -> Result<(),Message> {
        let blocker = lock!(self.0).blocker.clone();
        blocker.wait().await;
        let state = lock!(self.0);
        let ready = lock!(state.in_stage).is_ready();
        if ready && state.force_needed.is_needed() {
            *lock!(state.out_stage) = lock!(state.in_stage).clone();
            lock!(state.out_stage).make_report(&state.report);
        }
        Ok(())
    }

    pub(crate) fn apply_force(&self) {
        let commander = lock!(self.0).commander.clone();
        let self2 = self.clone();
        commander.add("force-applier", 0, None, None, Box::pin(async move { self2.force_applier().await }));
    }

    pub(crate) fn force_report(&self) {
        let state = lock!(self.0);
        state.force_needed.set();
    }

    pub(crate) fn set_stick(&self, stick: &str) {
        let state = lock!(self.0);
        let mut value =  lock!(state.in_stage);
        let changed = value.stick != Some(stick.to_string());
        value.stick = Some(stick.to_string());
        value.x = None;
        value.bp_per_screen = None;
        if changed {
            state.needed.set();
        }
    }

    pub(crate) fn set_position(&self, centre: Option<f64>, size: Option<f64>) {
        let state = lock!(self.0);
        let mut changed = false;
        if let Some(centre) = centre {
            changed |= lock!(state.in_stage).x != Some(centre);
            lock!(state.in_stage).x = Some(centre);    
        }
        if let Some(size) = size {
            changed |= lock!(state.in_stage).bp_per_screen != Some(size);
            lock!(state.in_stage).bp_per_screen = Some(size);
        }
        if changed {
            state.needed.set();
        }
    }
}
