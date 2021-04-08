use super::super::{ GPUSpec, Phase };
use super::program::{ Program, ProgramBuilder };
use crate::util::message::Message;
use web_sys::WebGlRenderingContext;

pub(crate) trait Source {
    fn cloned(&self) -> Box<dyn Source>;
    fn declare(&self, _spec: &GPUSpec, _phase: Phase) -> String { String::new() }
    fn statement(&self, _phase: Phase) -> String { String::new() }
    fn register(&self, builder: &mut ProgramBuilder) -> Result<(),Message> { Ok(()) }
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

    pub(crate) fn register(&self, builder: &mut ProgramBuilder) -> Result<(),Message> {
        for v in self.source.iter() {
            v.register(builder)?;
        }
        Ok(())
    }
}
