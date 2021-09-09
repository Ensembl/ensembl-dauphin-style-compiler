// build.rs
use std::process::Command;
use chrono::{DateTime, Utc};
use std::time::SystemTime;

fn main() {
    // taken from https://stackoverflow.com/questions/43753491/include-git-commit-hash-as-string-into-rust-program
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .unwrap();
    let git_hash = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
    /**/
    let now = SystemTime::now();
    let now: DateTime<Utc> = now.into();
    let now = now.to_rfc3339();
    println!("cargo:rustc-env=BUILD_TIME={}", now);
    /**/
    let hostname = hostname::get()
        .map(|s| s.into_string()).ok().transpose().ok().flatten().unwrap_or("hosntmae-unavailable".to_string());
    println!("cargo:rustc-env=BUILD_HOST={}",hostname);
}
