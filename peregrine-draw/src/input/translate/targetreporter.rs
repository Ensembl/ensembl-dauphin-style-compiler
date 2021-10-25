use std::sync::{Arc, Mutex};
use peregrine_data::DataMessage;
use peregrine_toolkit::{lock, sync::{blocker::{Blocker, Lockout}, needed::Needed}};
use crate::{PgCommanderWeb, run::report::Report, util::debounce::Debounce};

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
    force: bool,
    needed: Needed,         // pending update
    blocker: Blocker,       // block reports
    debounce: Debounce  // debounce
}

#[derive(Clone)]
pub struct TargetReporter(Arc<Mutex<TargetReporterState>>);

impl TargetReporter {
    async fn report_loop(&self) -> Result<(),DataMessage> {
        let needed = lock!(self.0).needed.clone();
        loop {
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
    }

    pub fn new(commander: &PgCommanderWeb, report: &Report) -> TargetReporter {
        let out_stage :Arc<Mutex<TargetLocation>> = Arc::new(Mutex::new(TargetLocation::empty()));
        let out_stage2 = out_stage.clone();
        let report2 = report.clone();
        let debounce = Debounce::new(commander,5000., move || { // XXX configurable
            lock!(out_stage2).make_report(&report2);
        });
        let out = TargetReporter(Arc::new(Mutex::new(TargetReporterState {
            report: report.clone(),
            out_stage,
            in_stage: Arc::new(Mutex::new(TargetLocation::empty())),
            needed: Needed::new(),
            blocker: Blocker::new(),
            force: false,
            debounce,
        })));
        let out2 = out.clone();
        commander.add("target-reporter", 0, None, None, Box::pin(async move { out2.report_loop().await }));
        out
    }

    pub fn lock_updates(&self) -> Lockout {
        lock!(self.0).blocker.lock()
    }

    fn force_report_ready(&self) {
        let state = lock!(self.0);
        *lock!(state.out_stage) = lock!(state.in_stage).clone();
        lock!(state.out_stage).make_report(&state.report);
    }

    pub fn force_report(&self) {
        let mut state = lock!(self.0);
        let ready= lock!(state.in_stage).is_ready();
        if ready {
            drop(state);
            self.force_report_ready();
        } else {
            state.force = true;
        }
    }

    pub fn set_stick(&self, stick: &str) {
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

    pub fn set_x(&self, x: f64) {
        let mut state = lock!(self.0);
        let changed = lock!(state.in_stage).x != Some(x);
        lock!(state.in_stage).x = Some(x);
        let ready= lock!(state.in_stage).is_ready();
        if state.force && ready {
            state.force = false;
            drop(state);
            self.force_report_ready();
        } else if changed {
            state.needed.set();
        }
    }

    pub fn set_bp(&self, bp: f64) {
        let mut state = lock!(self.0);
        let changed = lock!(state.in_stage).bp_per_screen != Some(bp);
        lock!(state.in_stage).bp_per_screen = Some(bp);
        let ready= lock!(state.in_stage).is_ready();
        if state.force && ready {
            state.force = false;
            drop(state);
            self.force_report_ready();
        } else if changed {
            state.needed.set();
        }
    }
}
