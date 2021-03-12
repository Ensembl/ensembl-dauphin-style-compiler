use std::sync::{ Arc, Mutex };
use js_sys::Date;
use web_sys::console;

pub enum PgConsoleLevel {
    Notice,
    Warn,
    Error
}

pub struct PgConsoleData {
    this_interval: f64,
    num_this_interval: u32,
    max_per_interval: u32,
    interval: f64
}

impl PgConsoleData {
    pub fn new(max_per_interval: u32, interval: f64) -> PgConsoleData {
        PgConsoleData {
            this_interval: 0.,
            num_this_interval: 0,
            max_per_interval,
            interval: interval * 1000.
        }
    }

    fn log(&self, level: PgConsoleLevel, msg: &str) {
        match level {
            PgConsoleLevel::Notice => console::log_1(&msg.to_string().into()),
            PgConsoleLevel::Warn => console::warn_1(&msg.to_string().into()),
            PgConsoleLevel::Error => console::error_1(&msg.to_string().into())
        }
    }

    fn interval(&self, a: f64) -> f64 {
        (a/self.interval).floor()
    }

    fn suppress(&mut self) -> bool {
        let now = self.interval(Date::now());
        if now.floor() > self.this_interval.floor() {
            if self.num_this_interval > self.max_per_interval {
                self.log(PgConsoleLevel::Notice,&format!("... and {} more messages in the last {}s",self.num_this_interval-self.max_per_interval,self.interval/1000.));
            }
            self.this_interval = now;
            self.num_this_interval = 0;
        }
        self.num_this_interval += 1;
        self.num_this_interval <= self.max_per_interval
    }

    pub fn message(&mut self, level: PgConsoleLevel, msg: &str) {
        if !self.suppress() {
            self.log(level,msg);
        }
    }
}

#[derive(Clone)]
pub struct PgConsoleWeb(Arc<Mutex<PgConsoleData>>);

impl PgConsoleWeb {
    pub fn new(max_per_interval: u32, interval: f64) -> PgConsoleWeb {
        PgConsoleWeb(Arc::new(Mutex::new(PgConsoleData::new(max_per_interval,interval))))
    }

    pub(crate) fn warn(&self, msg: &str) {
        self.0.lock().unwrap().message(PgConsoleLevel::Warn,msg);
    }

    pub(crate) fn error(&self, msg: &str) {
        self.0.lock().unwrap().message(PgConsoleLevel::Error,msg);
    }
}
