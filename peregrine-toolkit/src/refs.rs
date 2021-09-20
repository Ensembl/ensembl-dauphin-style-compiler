#[macro_export]
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