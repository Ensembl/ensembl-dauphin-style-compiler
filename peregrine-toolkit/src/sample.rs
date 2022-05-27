use std::{collections::HashMap, sync::{Arc, Mutex}};

use crate::{time::now, log};
use lazy_static::lazy_static;

#[cfg(debug_sampler)]
use crate::lock;

const REPORT_SEC : u64 = 2;

pub struct Sampler {
    name: String,
    cur_string: Option<String>,
    cur_number: Option<u64>,
    strings: HashMap<String,u64>,
    numbers: HashMap<u64,u64>,
    last_seen: Option<u64>,
    timers: HashMap<String,u64>
}

impl Sampler {
    pub fn new(name: &str) -> Sampler {
        Sampler {
            name: name.to_string(),
            cur_string: None,
            cur_number: None,
            strings: HashMap::new(),
            numbers: HashMap::new(),
            last_seen: None,
            timers: HashMap::new()
        }
    }

    fn sample(&mut self, ms_passed: u64) {
        if let Some(value) = &self.cur_string {
            *self.strings.entry(value.clone()).or_insert(0) += ms_passed;
        }
        if let Some(value) = &self.cur_number {
            *self.numbers.entry(*value).or_insert(0) += ms_passed;
        }
    }

    fn report(&mut self) {
        let mut values = vec![];
        for (k,v) in &self.strings {
            values.push(format!("{}: {}",k,v));
        }
        for (k,v) in &self.numbers {
            values.push(format!("{}: {}",k,v));
        }
        log!("{}: {}",&self.name,values.join(", "));
        self.strings.clear();
        self.numbers.clear();
    }

    fn try_sample(&mut self) {
        let now = now() as u64;
        if let Some(last_seen) = self.last_seen {
            if now > last_seen {
                self.sample(now-last_seen);
                if now / 1000 / REPORT_SEC != last_seen / 1000 / REPORT_SEC {
                    self.report();
                }
                self.last_seen = Some(now);
            }
        } else {
            self.last_seen = Some(now);
        }
    }

    pub fn set_string(&mut self, value: String) {
        self.try_sample();
        self.cur_string = Some(value);
    }

    pub fn set_number(&mut self, value: u64) {
        self.try_sample();
        self.cur_number = Some(value);
    }

    pub fn timer_start(&mut self, name: &str) {
        let now = now() as u64;
        self.timers.insert(name.to_string(),now);
    }

    pub fn timer_end(&mut self, name: &str) {
        let now = now() as u64;
        if let Some(start) = self.timers.get(name) {
            *self.strings.entry(name.to_string()).or_insert(0) += now - start;
        }
    }
}

lazy_static! {
    pub static ref SAMPLER : Arc<Mutex<Sampler>> = Arc::new(Mutex::new(Sampler::new("GLOBAL")));
}    

#[macro_export]
macro_rules! sample {
    ($num:expr) => {
        #[cfg(debug_sampler)]
        $crate::lock!($crate::sample::SAMPLER).set_number($num)
    };
}

#[macro_export]
macro_rules! sample_str {
    ($name:expr) => {
        #[cfg(debug_sampler)]
        $crate::lock!($crate::sample::SAMPLER).set_string($name)
    };
}

#[macro_export]
macro_rules! timer_start {
    ($name:expr) => {
        #[cfg(debug_sampler)]
        $crate::lock!($crate::sample::SAMPLER).timer_start($name)
    };
}

#[macro_export]
macro_rules! timer_end {
    ($name:expr) => {
        #[cfg(debug_sampler)]
        $crate::lock!($crate::sample::SAMPLER).timer_end($name)
    };
}

lazy_static! {
    pub static ref MARK_IDS : Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
}    

#[cfg(debug_sampler)]
pub struct RegionStart(String);

#[cfg(not(debug_sampler))]
pub struct RegionStart();

#[cfg(debug_sampler)]
fn mark_id() -> RegionStart {
    let mut id = lock!(MARK_IDS);
    *id += 1;
    RegionStart(format!("m{}",id))
}

#[cfg(debug_sampler)]
pub fn region_start() -> RegionStart {
    let window = web_sys::window().unwrap();
    let performance = window.performance().unwrap();
    let name = mark_id();
    performance.mark(&name.0).ok();
    name
}

#[cfg(debug_sampler)]
pub fn region_end(name: &str, start: RegionStart) {
    let window = web_sys::window().unwrap();
    let performance = window.performance().unwrap();
    let end = mark_id();
    performance.mark(&end.0).ok();
    performance.measure_with_start_mark_and_end_mark(name, &start.0, &end.0);
}

#[cfg(not(debug_sampler))]
pub fn region_start() -> RegionStart { RegionStart() }

#[cfg(not(debug_sampler))]
pub fn region_end(_name: &str, _start: RegionStart) {}
