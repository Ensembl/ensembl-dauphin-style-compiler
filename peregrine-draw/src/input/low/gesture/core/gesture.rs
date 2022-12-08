use std::sync::{ Arc, Mutex };
use peregrine_toolkit::{lock};
use peregrine_toolkit_async::sync::blocker::Lockout;
use crate::Message;
use crate::input::low::gesture::node::unknown::Unknown;
use crate::input::low::lowlevel::LowLevelState;
use crate::input::low::modifiers::Modifiers;
use crate::input::low::pointer::PointerConfig;
use crate::input::translate::targetreporter::TargetReporter;
use crate::run::CursorCircumstance;
use crate::webgl::global::WebGlGlobal;
use super::gesturenode::{GestureNode, GestureNodeImpl };
use super::finger::{OneOrTwoFingers};
use super::cursor::CursorHandle;
use super::transition::{GestureNodeTransition, TimerHandleSource, TimerHandle};

pub(crate) struct GestureNodeState {
    pub(crate) lowlevel: LowLevelState,
    pub(crate) config: Arc<PointerConfig>,
    pub(crate) gl: Arc<Mutex<WebGlGlobal>>,
    pub(crate) initial_modifiers: Modifiers
}

pub(crate) struct GestureState {
    state: GestureNodeState,
    fingers: OneOrTwoFingers,
    mode: GestureNode,
    timer_handles: TimerHandleSource,
    #[allow(unused)] // keep as guard
    cursor: Option<CursorHandle>,
    #[allow(unused)]
    intention_lockout: Option<Lockout>,
    target_reporter: TargetReporter
}

impl GestureState {
    fn new(lowlevel: &LowLevelState, gl: &Arc<Mutex<WebGlGlobal>>, config: &Arc<PointerConfig>, primary: (f64,f64), secondary: Option<(f64,f64)>, target_reporter: &TargetReporter) -> Result<GestureState,Message> {
        Ok(GestureState {
            state: GestureNodeState {
                lowlevel: lowlevel.clone(),
                config: config.clone(),
                gl: gl.clone(),
                initial_modifiers: lowlevel.modifiers()
            },
            fingers: OneOrTwoFingers::new(primary,secondary),
            mode: GestureNode::new(Unknown::new()), // Never actually used but better than placeholder
            timer_handles: TimerHandleSource::new(),
            cursor: None,
            intention_lockout: Some(target_reporter.lock_updates()),
            target_reporter: target_reporter.clone(),
        })
    }

    pub(super) fn add_timer(&mut self, time: f64, twin: &Arc<Mutex<GestureState>>, handle: TimerHandle) {
        let twin = twin.clone();
        let timers = self.timer_handles.clone();
        self.state.lowlevel.timer(time, move || {
            let mut transition = GestureNodeTransition::new(&timers);
            if timers.is_valid(&handle) {
                let twin_lock = &mut *lock!(twin);
                let (mode,mut state,mut primary) = 
                    (&mut twin_lock.mode,&mut twin_lock.state,&mut twin_lock.fingers);
                // TODO error propagation
                mode.timeout(&mut transition,&mut state,&mut primary,handle);
                drop(twin_lock);
            }
            transition.apply(&twin);
        });
    }

    pub(super) fn update_cursor(&mut self, cursor: &CursorCircumstance) {
        self.cursor = Some(self.state.lowlevel.set_cursor(cursor));
    }

    pub(super) fn set_mode(&mut self, mode: GestureNode) -> Result<GestureNodeTransition,Message> {
        self.mode = mode;
        let mut transition = GestureNodeTransition::new(&self.timer_handles);
        self.timer_handles.invalidate();
        self.mode.init(&mut transition,&mut self.state,&mut self.fingers)?;
        Ok(transition)
    }

    fn drag_continue(&mut self, primary: (f64,f64), secondary: Option<(f64,f64)>) -> Result<GestureNodeTransition,Message> {
        self.fingers.set(primary,secondary);
        let mut transition = GestureNodeTransition::new(&self.timer_handles);
        self.mode.continues(&mut transition,&mut self.state,&mut self.fingers)?;
        Ok(transition)
    }

    fn drag_finished(&mut self, primary: (f64,f64), secondary: Option<(f64,f64)>) -> Result<bool,Message> {
        self.fingers.set(primary,secondary);
        let mut transition = GestureNodeTransition::new(&self.timer_handles);
        self.mode.continues(&mut transition,&mut self.state,&mut self.fingers)?;
        self.cursor = None;
        self.intention_lockout = None;
        let valid = self.mode.finished(&mut self.state,&mut self.fingers)?;
        if valid { self.target_reporter.force_report() }
        Ok(valid)
    }
}

pub(crate) struct Gesture(Arc<Mutex<GestureState>>);

impl Gesture {
    pub(crate) fn new(config: &Arc<PointerConfig>, lowlevel: &LowLevelState, gl: &Arc<Mutex<WebGlGlobal>>, primary: (f64,f64), secondary: Option<(f64,f64)>, target_reporter: &TargetReporter) -> Result<Gesture,Message> {
        let inner = Arc::new(Mutex::new(GestureState::new(lowlevel,gl,config,primary,secondary,target_reporter)?));
        let transition = lock!(inner).set_mode(GestureNode::new(Unknown::new()))?;
        transition.apply(&inner)?;
        let mut out = Gesture(inner);
        out.drag_continue(primary,secondary)?;
        Ok(out)
    }

    pub(crate) fn drag_continue(&mut self, primary: (f64,f64), secondary: Option<(f64,f64)>) -> Result<(),Message> {
        let transition = lock!(self.0).drag_continue(primary,secondary)?;
        transition.apply(&self.0)?;
        Ok(())
    }

    pub(crate) fn drag_finished(&mut self, primary: (f64,f64), secondary: Option<(f64,f64)>) -> Result<bool,Message> {
        self.0.lock().unwrap().drag_finished(primary,secondary)
    }
}
