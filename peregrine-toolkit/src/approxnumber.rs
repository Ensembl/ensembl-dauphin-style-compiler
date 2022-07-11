use std::hash::Hash;

pub struct ApproxNumber(pub f64,pub i32);

impl Hash for ApproxNumber {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let log = (self.0).abs().log10().floor() as i32;
        let mul = (10_f64).powi(self.1-log-1);
        let x = (self.0*mul).round() as i64;
        log.hash(state);
        x.hash(state);
    }
}
