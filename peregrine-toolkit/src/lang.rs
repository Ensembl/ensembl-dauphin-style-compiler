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

#[macro_export]
macro_rules! ubail {
    ($value:expr,$bailer:expr) => {
        if let Some(x) = $value { x } else { return $bailer }
    }
}