use super::super::{ GPUSpec, Phase };

pub(crate) trait Runtime {

}

pub(crate) trait Source {
    fn declare(&self, _spec: &GPUSpec, _phase: Phase) -> String { String::new() }
    fn statement(&self, _phase: Phase) -> String { String::new() }
    fn to_binary(&self) -> Box<dyn Runtime>;
}

pub(crate) struct SourceInstrs {
    source: Vec<Box<dyn Source>>
}

impl SourceInstrs {
    pub fn new(source: Vec<Box<dyn Source>>) -> SourceInstrs {
        SourceInstrs {
            source
        }
    }

    pub fn merge(&mut self, mut other: SourceInstrs) {
        self.source.extend(other.source.drain(..));
    }

    fn declare(&self, gpuspec: &GPUSpec, phase: Phase) -> String {
        let mut out = String::new();
        for stmt in self.source.iter() {
            out += &stmt.declare(gpuspec,phase)[..];
        }
        out
    }

    fn statements(&self, phase: Phase) -> String {
        let mut out = String::new();
        for v in self.source.iter() {
            out += &v.statement(phase)[..];
        }
        out
    }

    pub fn serialise(&self, gpuspec: &GPUSpec, phase: Phase) -> String {
        format!("{}\n\nvoid main() {{\n{}\n}}",
            self.declare(gpuspec,phase),
            self.statements(phase))
    }    
}
