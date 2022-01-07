// build.rs
use chrono::{DateTime, Utc};
use std::time::SystemTime;

fn main() {
    /**/
    let now = SystemTime::now();
    let now: DateTime<Utc> = now.into();
    let now = now.to_rfc3339();
    println!("cargo:rustc-env=BUILD_TIME={}", now);
    println!("cargo:rerun-if-changed=..");
}
