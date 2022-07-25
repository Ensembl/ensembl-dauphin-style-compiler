// build.rs
use chrono::{DateTime, Utc};
use std::{time::SystemTime, env, path::Path, fs::File, io::Write};

fn main() {
    /**/
    let now = SystemTime::now();
    let now: DateTime<Utc> = now.into();
    let now = now.to_rfc3339();
    println!("cargo:rustc-env=BUILD_TIME={}", now);
    println!("cargo:rerun-if-changed=..");
    /**/
    let out_dir = env::var("OUT_DIR").expect("No out dir");
    let dest_path = Path::new(&out_dir).join("env.rs");
    let mut f = File::create(&dest_path).expect("Could not create file");

    let force_dpr = option_env!("FORCE_DPR").map(|f| str::parse::<f32>(f))
        .transpose()
        .expect("Could not parse FORCE_DPR");
    write!(&mut f, "const FORCE_DPR: Option<f32> = {:?};", force_dpr)
        .expect("Could not write file");
    println!("cargo:rerun-if-env-changed=FORCE_DPR");
}
