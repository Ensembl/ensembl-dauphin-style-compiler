#[cfg(not(test))]
use js_sys::Date;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub fn now() -> f64 {
    Date::now()    
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
pub fn now() -> f64 {
    0.
}
