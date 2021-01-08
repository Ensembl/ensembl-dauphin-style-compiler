use super::super::{ GPUSpec, Phase };

pub(crate) trait Source {
    fn declare(&self, _spec: &GPUSpec, _phase: Phase) -> String { String::new() }
    fn statement(&self, _spec: &GPUSpec, _phase: Phase) -> String { String::new() }
}
