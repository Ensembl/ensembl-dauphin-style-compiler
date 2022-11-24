use std::iter::Sum;

use crate::LoadMode;

#[cfg(debug_canvasstore)]
#[derive(Clone)]
pub(crate) struct OriginStats(u64,u64);

#[cfg(debug_canvasstore)]
impl OriginStats {
    pub(crate) fn empty() -> OriginStats { OriginStats(0,0) }

    pub(crate) fn new(mode: &LoadMode) -> OriginStats {
        let mut out = OriginStats(0,0);
        if mode.high_priority() { out.0 += 1; } else { out.1 += 1 }
        out
    }

    pub(crate) fn merge(&mut self, other: &OriginStats) {
        self.0 += other.0;
        self.1 += other.1;
    }
    
    pub(crate) fn report(&self) {
        use peregrine_toolkit::log;

        if self.0+self.1 > 0 && (self.0+self.1) % 100 == 0 {
            let perc = self.1 * 100 / (self.0+self.1);
            log!("shape cache hit rate {}%",perc);    
        }
    }
}

#[cfg(debug_canvasstore)]
impl<'a> Sum<&'a OriginStats> for OriginStats {
    fn sum<I: Iterator<Item = &'a Self>>(mut iter: I) -> Self {
        let mut out = OriginStats(0,0);
        while let Some(OriginStats(a,b)) = iter.next() {
            out.0 += a;
            out.1 += b;
        }
        out
    }
}

#[cfg(not(debug_canvasstore))]
#[derive(Clone)]
pub(crate) struct OriginStats;

#[cfg(not(debug_canvasstore))]
impl OriginStats {
    pub(crate) fn empty() -> OriginStats { OriginStats }
    pub(crate) fn new(_mode: &LoadMode) -> OriginStats { OriginStats }
    pub(crate) fn merge(&mut self, _other: &OriginStats) {}    
    pub(crate) fn report(&self) {}
}

#[cfg(not(debug_canvasstore))]
impl<'a> Sum<&'a OriginStats> for OriginStats {
    fn sum<I: Iterator<Item = &'a Self>>(_iter: I) -> Self { OriginStats }
}
