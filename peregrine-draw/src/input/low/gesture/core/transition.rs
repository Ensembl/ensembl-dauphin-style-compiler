use std::sync::{Arc, Mutex};
use peregrine_toolkit::lock;
use crate::{run::CursorCircumstance, Message};
use super::{gesturenode::GestureNode, gesture::GestureState};

#[derive(Clone,Hash,PartialEq,Eq)]
pub(crate) struct TimerHandle(u64);

#[derive(Clone)]
pub(super) struct TimerHandleSource {
    min_valid_timer_handle: Arc<Mutex<u64>>,
    next_timer_handle: Arc<Mutex<u64>>
}

impl TimerHandleSource {
    pub(super) fn new() -> TimerHandleSource {
        TimerHandleSource {
            min_valid_timer_handle: Arc::new(Mutex::new(0)),
            next_timer_handle: Arc::new(Mutex::new(0))
        }
    }

    pub(super) fn next(&self) -> TimerHandle {
        let mut next = lock!(self.next_timer_handle);
        let handle = TimerHandle(*next);
        *next += 1;
        handle
    }

    pub(super) fn is_valid(&self, handle: &TimerHandle) -> bool {
        let min = lock!(self.min_valid_timer_handle);
        handle.0 >= *min
    }

    pub(super) fn invalidate(&self) {
        let next = lock!(self.next_timer_handle);
        let mut min = lock!(self.min_valid_timer_handle);
        *min = *next;
    }
}

pub(crate) struct GestureNodeTransition {
    new_mode: Option<GestureNode>,
    cursor: Option<CursorCircumstance>,
    timer_handles: TimerHandleSource,
    new_timers: Vec<(TimerHandle,f64)>
}

impl GestureNodeTransition {
    pub(super) fn new(timer_handles: &TimerHandleSource) -> GestureNodeTransition {
        GestureNodeTransition { 
            new_mode: None,
            cursor: None,
            timer_handles: timer_handles.clone(),
            new_timers: vec![]
        }
    }

    pub(crate) fn new_mode(&mut self, mode: GestureNode) {
        self.new_mode = Some(mode);
    }

    pub(crate) fn set_cursor(&mut self, cursor: CursorCircumstance) {
        self.cursor = Some(cursor);
    }

    pub(super) fn apply(mut self, inner: &Arc<Mutex<GestureState>>) -> Result<(),Message> {
        let twin = inner.clone();
        let mut state = lock!(inner);
        let mut new_transition = None;
        if let Some(cursor) = &self.cursor {
            state.update_cursor(cursor);
        }
        if let Some(new_mode) = self.new_mode {
            new_transition = Some(state.set_mode(new_mode)?);
        }
        for (handle,timeout) in self.new_timers.drain(..) {
            state.add_timer(timeout,&twin,handle);
        }
        drop(state);
        if let Some(new_transition) = new_transition {
            new_transition.apply(inner)?;
        }
        Ok(())
    }

    pub(crate) fn add_timer(&mut self, time: f64) -> TimerHandle {
        let handle = self.timer_handles.next();
        self.new_timers.push((handle.clone(),time));
        handle
    }
}
