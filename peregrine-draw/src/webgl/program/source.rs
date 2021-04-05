use super::super::{ GPUSpec, Phase };
use super::program::Program;
use crate::util::message::Message;

pub(crate) trait Source {
    fn cloned(&self) -> Box<dyn Source>;
    fn declare(&self, _spec: &GPUSpec, _phase: Phase) -> String { String::new() }
    fn statement(&self, _phase: Phase) -> String { String::new() }
    fn build(&mut self, _program: &mut Program) -> Result<(),Message> { Ok(()) }
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
    pub(crate) fn new(source: Vec<Box<dyn Source>>) -> SourceInstrs {
        SourceInstrs {
            source
        }
    }

    pub(crate) fn merge(&mut self, mut other: SourceInstrs) {
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

    pub(crate) fn serialise(&self, gpuspec: &GPUSpec, phase: Phase) -> String {
        format!("{}\n\nvoid main() {{\n{}\n}}",
            self.declare(gpuspec,phase),
            self.statements(phase))
    }

    pub(crate) fn build(&mut self, program: &mut Program) -> Result<(),Message> {
        for v in self.source.iter_mut() {
            v.build(program)?;
        }
        Ok(())
    }
}
