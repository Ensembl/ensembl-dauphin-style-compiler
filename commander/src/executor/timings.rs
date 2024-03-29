use ordered_float::OrderedFloat;
use crate::integration::integration::SleepQuantity;
use crate::integration::reentering::ReenteringIntegration;
use super::taskcontainer::TaskContainer;
use super::taskcontainerhandle::TaskContainerHandle;
use super::timerset::TimerSet;

enum TimerType {
    Task(TaskContainerHandle),
    Standalone
}


fn gate_fn<'a>(tasks: &'a TaskContainer) -> Box<dyn Fn(&TimerType) -> bool + 'a> {
    Box::new(move |kind: &TimerType| { 
        match kind {
            TimerType::Task(handle) => tasks.get(handle).is_some(),
            TimerType::Standalone => true
        }
    })
}

pub(crate) struct ExecutorTimings {
    integration: ReenteringIntegration,
    timers: TimerSet<OrderedFloat<f64>,TimerType>,
    ticks: TimerSet<u64,TimerType>,
    tick_index: u64
}

impl ExecutorTimings {
    pub(crate) fn new(integration: &ReenteringIntegration) -> ExecutorTimings {
        ExecutorTimings {
            integration: integration.clone(),
            timers: TimerSet::new(),
            ticks: TimerSet::new(),
            tick_index: 0,
        }
    }

    pub(crate) fn run_timers(&self, tasks: &TaskContainer) {
        let now = self.integration.current_time();
        let gate = gate_fn(tasks);
        self.timers.run(OrderedFloat(now),&gate);
        self.ticks.run(self.tick_index,&gate);
    }

    pub(crate) fn run_ticks(&self, tasks: &TaskContainer) {
        let gate = gate_fn(tasks);
        self.ticks.run(self.tick_index,&gate);
    }

    pub(crate) fn advance_tick(&mut self) {
        self.tick_index += 1;
    }

    pub(crate) fn get_tick_index(&self) -> u64 { self.tick_index }

    pub(crate) fn add_timer(&mut self, handle: &TaskContainerHandle, timeout: f64, callback: Box<dyn FnOnce() + 'static>) {
        let now = self.integration.current_time();
        self.timers.add(TimerType::Task(handle.clone()),OrderedFloat(now+timeout),callback);
    }

    pub(crate) fn add_standalone_timer(&mut self, timeout: f64, callback: Box<dyn FnOnce() + 'static>) {
        let now = self.integration.current_time();
        self.timers.add(TimerType::Standalone,OrderedFloat(now+timeout),callback);
    }

    pub(crate) fn add_tick(&mut self, handle: &TaskContainerHandle, tick: u64, callback: Box<dyn FnOnce() + 'static>) {
        self.ticks.add(TimerType::Task(handle.clone()),tick,callback);
    }

    pub(crate) fn calculate_sleep(&self, now: f64) -> SleepQuantity {
        if self.ticks.len() > 0 {
            SleepQuantity::None
        } else if let Some(timer) = self.timers.min() {
            SleepQuantity::Time(timer.0-now)
        } else {
            SleepQuantity::Forever
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::{ Arc, Mutex };
    use crate::executor::executor::Executor;
    use crate::integration::testintegration::TestIntegration;
    use crate::task::runconfig::RunConfig;

    #[test]
    pub fn test_control_timers() {
        /* setup */
        let mut integration = TestIntegration::new();
        let mut x = Executor::new(integration.clone());
        let cfg = RunConfig::new(None,2,None);
        let ctx = x.new_agent(&cfg,"test");
        let tc = x.add(async {},ctx);        
        /* test */
        let shared = Arc::new(Mutex::new(false));
        let shared2 = shared.clone();
        tc.get_agent().add_timer(1.,move || { *shared2.lock().unwrap() = true; });
        x.service();
        integration.set_time(0.5);
        x.get_tasks().run_timers(x.get_timings());
        assert!(!*shared.lock().unwrap());
        integration.set_time(1.5);
        x.get_tasks().run_timers(x.get_timings());
        assert!(*shared.lock().unwrap());
    } 
}