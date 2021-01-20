use super::super::{ GPUSpec, Phase };
use super::program::Program;

pub(crate) trait Source {
    fn cloned(&self) -> Box<dyn Source>;
    fn declare(&self, _spec: &GPUSpec, _phase: Phase) -> String { String::new() }
    fn statement(&self, _phase: Phase) -> String { String::new() }
    fn build(&mut self, program: &mut Program) -> anyhow::Result<()> { Ok(()) }
}

pub(crate) struct SourceInstrs {
    source: Vec<Box<dyn Source>>
}

impl Clone for SourceInstrs {
    fn clone(&self) -> SourceInstrs {
        SourceInstrs {
            source: self.source.iter().map(|x| x.cloned()).collect()
        }
    }
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

    pub fn build(&mut self, program: &mut Program) -> anyhow::Result<()> {
        for v in self.source.iter_mut() {
            v.build(program)?;
        }
        Ok(())
    }
}
