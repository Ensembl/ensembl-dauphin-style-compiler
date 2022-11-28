#[macro_export]
macro_rules! map {
    ($($key:expr => $value:expr),*) => {
        {
            use std::collections::HashMap;

            let mut out = HashMap::new();
            $(
                out.insert($key,$value);
            )*
            out
        }
    }
}

pub use map;