use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use crate::lock;

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone,Copy)]
pub enum Severity {
    Notice,
    Warning,
    Error
}

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Copy,Clone)]
pub enum Verbosity {
    Noisy,
    Normal,
    Quiet
}

impl Verbosity {
    pub fn from_string(str: &str) -> Option<Verbosity> {
        match str {
            "quiet" => Some(Verbosity::Quiet),
            "noisy" => Some(Verbosity::Noisy),
            "normal" => Some(Verbosity::Normal),
            _ => None
        }
    }

    fn level(&self) -> usize {
        match self {
            Verbosity::Quiet => 0,
            Verbosity::Normal => 1,
            Verbosity::Noisy => 2
        }
    }
}

#[cfg(not(any(console_quiet,console_noisy)))]
static DEFAULT_VERBOSITY : Verbosity = Verbosity::Normal;
#[cfg(console_quiet)]
static DEFAULT_VERBOSITY : Verbosity = Verbosity::Quiet;
#[cfg(console_noisy)]
static DEFAULT_VERBOSITY : Verbosity = Verbosity::Noisy;

lazy_static! {
    static ref VERBOSITY : Arc<Mutex<Verbosity>> = Arc::new(Mutex::new(DEFAULT_VERBOSITY));
    static ref PRINTER : Arc<Mutex<Option<Box<dyn FnMut(&Severity,&str) + 'static + Send>>>> = Arc::new(Mutex::new(None));
}

pub fn set_verbosity(verbosity: &Option<Verbosity>) {
    if let Some(verbosity) = verbosity {
        *lock!(VERBOSITY) = verbosity.clone();
    }
}

pub fn set_printer<F>(cb: F) where F: FnMut(&Severity,&str) + 'static + Send {
    *lock!(PRINTER) = Some(Box::new(cb));
}

/* Don't call directly, use macros */
pub fn print(verbosity: &Verbosity, severity: &Severity, message: &str) {
    if verbosity.level() > lock!(VERBOSITY).level() { return; }
    if let Some(printer) = lock!(PRINTER).as_mut() {
        printer(severity,message);
    }
}

#[macro_export]
macro_rules! do_log {
    ($verb:tt,$sev:tt,$($arg:tt)*) => {
        use $crate::console::*;
        (print(&Verbosity::$verb,&Severity::$sev,&std::format!($($arg)*)))
    }
}

#[macro_export]
macro_rules! debug_do_log {
    ($verb:tt,$sev:tt,$($arg:tt)*) => {
        use $crate::console::*;
        #[cfg(debug_assertions)]
        (print(&Verbosity::$verb,&Severity::$sev,&std::format!($($arg)*)))
    }
}

#[macro_export]
macro_rules! log { ($($arg:tt)*) => { $crate::do_log!(Normal,Notice,$($arg)*); } }
#[macro_export]
macro_rules! log_important { ($($arg:tt)*) => { $crate::do_log!(Quiet,Notice,$($arg)*); } }
#[macro_export]
macro_rules! log_extra { ($($arg:tt)*) => { $crate::do_log!(Noisy,Notice,$($arg)*); } }
#[macro_export]
macro_rules! warn { ($($arg:tt)*) => { $crate::do_log!(Normal,Warning,$($arg)*); } }
#[macro_export]
macro_rules! warn_important { ($($arg:tt)*) => { $crate::do_log!(Quiet,Warning,$($arg)*); } }
#[macro_export]
macro_rules! warn_extra { ($($arg:tt)*) => { $crate::do_log!(Noisy,Warning,$($arg)*); } }
#[macro_export]
macro_rules! error { ($($arg:tt)*) => { $crate::do_log!(Normal,Error,$($arg)*); } }
#[macro_export]
macro_rules! error_important { ($($arg:tt)*) => { $crate::do_log!(Quiet,Error,$($arg)*); } }
#[macro_export]
macro_rules! error_extra { ($($arg:tt)*) => { $crate::do_log!(Noisy,Error,$($arg)*); } }

#[macro_export]
macro_rules! debug_log { ($($arg:tt)*) => { $crate::debug_do_log!(Normal,Notice,$($arg)*); } }
#[macro_export]
macro_rules! debug_log_important { ($($arg:tt)*) => { $crate::debug_do_log!(Quiet,Notice,$($arg)*); } }
#[macro_export]
macro_rules! debug_log_extra { ($($arg:tt)*) => { $crate::debug_do_log!(Noisy,Notice,$($arg)*); } }
#[macro_export]
macro_rules! debug_warn { ($($arg:tt)*) => { $crate::debug_do_log!(Normal,Warning,$($arg)*); } }
#[macro_export]
macro_rules! debug_warn_important { ($($arg:tt)*) => { $crate::debug_do_log!(Quiet,Warning,$($arg)*); } }
#[macro_export]
macro_rules! debug_warn_extra { ($($arg:tt)*) => { $crate::debug_do_log!(Noisy,Warning,$($arg)*); } }
#[macro_export]
macro_rules! debug_error { ($($arg:tt)*) => { $crate::debug_do_log!(Normal,Error,$($arg)*); } }
#[macro_export]
macro_rules! debug_error_important { ($($arg:tt)*) => { $crate::debug_do_log!(Quiet,Error,$($arg)*); } }
#[macro_export]
macro_rules! debug_error_extra { ($($arg:tt)*) => { $crate::debug_do_log!(Noisy,Error,$($arg)*); } }
