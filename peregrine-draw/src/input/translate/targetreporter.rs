use std::sync::{Arc, Mutex};
use peregrine_data::DataMessage;
use peregrine_toolkit::{lock, sync::{blocker::{Blocker, Lockout}, needed::Needed}};
use crate::{PgCommanderWeb, stage::stage::ReadStage, util::monostable::{Monostable}};

/* Lockable, debounced intention reoprted
 */

fn report(stage: &Arc<Mutex<Option<ReadStage>>>) -> bool {
    if let Some(stage) = lock!(stage).take() {
        if stage.ready() {
            use web_sys::console;
            console::log_1(&format!("{:?}/{}/{}",stage.stick(),stage.x().position().ok().unwrap(),stage.x().bp_per_screen().ok().unwrap()).into());                
            return true;
        }
    }
    false
}

struct TargetReporterState {
    in_stage: Arc<Mutex<Option<ReadStage>>>,  // latest report
    out_stage: Arc<Mutex<Option<ReadStage>>>, // ready to send
    force: bool,
    needed: Needed,         // pending update
    blocker: Blocker,       // block reports
    monostable: Monostable  // debounce
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
            // XXX non-critical race condition but examine all uses of Blocker
            *lock!(state.out_stage) = lock!(state.in_stage).clone();
            state.monostable.set();
            drop(state);
        }
    }

    pub fn new(commander: &PgCommanderWeb) -> TargetReporter {
        let out_stage :Arc<Mutex<Option<ReadStage>>> = Arc::new(Mutex::new(None));
        let out_stage2 = out_stage.clone();
        let monostable = Monostable::new(commander,5000., move || { // XXX configurable
            report(&out_stage2);
        });
        let out = TargetReporter(Arc::new(Mutex::new(TargetReporterState {
            out_stage,
            in_stage: Arc::new(Mutex::new(None)),
            needed: Needed::new(),
            blocker: Blocker::new(),
            force: false,
            monostable
        })));
        let out2 = out.clone();
        commander.add("target-reporter", 1, None, None, Box::pin(async move { out2.report_loop().await }));
        out
    }

    pub fn lock_updates(&self) -> Lockout {
        lock!(self.0).blocker.lock()
    }

    pub fn force_report(&self) {
        let mut state = lock!(self.0);
        if lock!(state.in_stage).is_some() {
            *lock!(state.out_stage) = lock!(state.in_stage).clone();
        }
        if !report(&state.out_stage) {
            state.force = true;
        }
    }

    pub fn stage(&self, stage: &ReadStage) {
        let mut state = lock!(self.0);
        *state.in_stage.lock().unwrap() = Some(stage.clone());
        if state.force {
            state.force = false;
            drop(state);
            self.force_report();
        } else {
            state.needed.set()
        }
    }
}
