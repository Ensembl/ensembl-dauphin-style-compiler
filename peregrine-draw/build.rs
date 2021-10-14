// build.rs
use std::process::Command;
use chrono::{DateTime, Utc};
use std::time::SystemTime;

fn command(command: &str, args: &[&str]) -> String {
    let output = Command::new(command).args(args).output().unwrap();
    String::from_utf8(output.stdout).unwrap()
}

fn main() {
    // taken from https://stackoverflow.com/questions/43753491/include-git-commit-hash-as-string-into-rust-program
    let git_hash = command("git",&["rev-parse","HEAD"]);
    let git_tag = command("git",&["describe","--exact-match","HEAD"]);
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
    println!("cargo:rustc-env=GIT_TAG={}", git_tag);
    /**/
    let now = SystemTime::now();
    let now: DateTime<Utc> = now.into();
    let now = now.to_rfc3339();
    println!("cargo:rustc-env=BUILD_TIME={}", now);
    /**/
    let hostname = hostname::get()
        .map(|s| s.into_string()).ok().transpose().ok().flatten().unwrap_or("hostname-unavailable".to_string());
    println!("cargo:rustc-env=BUILD_HOST={}",hostname);
    println!("cargo:rerun-if-changed=..");
}
