use std::sync::{ Arc, Mutex };
use crate::input::{InputEvent, low::{self, lowlevel::LowLevelInput}};

pub enum PullState {
    None,
    Left,
    Right
}

pub struct PhysicsState {
    pull_state: PullState
}

impl PhysicsState {
    fn new() -> PhysicsState {
        PhysicsState {
            pull_state: PullState::None
        }
    }
}

#[derive(Clone)]
pub struct Physics {
    state: Arc<Mutex<PhysicsState>>
}

impl Physics {
    fn incoming_event(&self, event: &InputEvent) {
        use web_sys::console;
        console::log_1(&format!("event: {:?}",event).into());
    }

    pub fn new(low_level: &mut LowLevelInput) -> Physics {
        let out = Physics {
            state: Arc::new(Mutex::new(PhysicsState::new()))
        };
        let out2 = out.clone();
        low_level.distributor_mut().add(move |e| out2.incoming_event(e));
        out
    }
}