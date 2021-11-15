
#[macro_export]
#[cfg(debug_assertions)]
macro_rules! lock {
    ($x: expr) => {{
        match $x.lock() {
            Ok(v) => v,
            Err(_) => {
                panic!("ENSEMBL ERROR LOCATION {}/{}/{}",file!(),line!(),column!());
            }
        }
    }}
}

#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! lock {
    ($x: expr) => {{
        $x.lock().unwrap()
    }}
}
