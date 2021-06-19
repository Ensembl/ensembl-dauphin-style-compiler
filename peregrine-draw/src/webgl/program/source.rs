use std::collections::HashSet;

use super::super::{ GPUSpec, Phase };
use super::program::{ ProgramBuilder };
use crate::util::message::Message;

pub(crate) trait Source {
    fn cloned(&self) -> Box<dyn Source>;
    fn declare(&self, _spec: &GPUSpec, _phase: Phase, _flags: &HashSet<String>) -> String { String::new() }
    fn statement(&self, _phase: Phase, _flags: &HashSet<String>) -> String { String::new() }
    fn register(&self, _builder: &mut ProgramBuilder, _flags: &HashSet<String>) -> Result<(),Message> { Ok(()) }
    fn set_flags(&self, _flags: &mut HashSet<String>) {}
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

    pub(crate) fn get_flags(&self) -> HashSet<String> {
        let mut flags = HashSet::new();
        for stmt in self.source.iter() {
            stmt.set_flags(&mut flags);
        }
        return flags;
    }

    pub(super) fn declare(&self, gpuspec: &GPUSpec, phase: Phase, flags: &HashSet<String>) -> String {
        let mut out = String::new();
        for stmt in self.source.iter() {
            out += &stmt.declare(gpuspec,phase,flags)[..];
        }
        out
    }

    pub(super) fn statement(&self, phase: Phase, flags: &HashSet<String>) -> String {
        let mut out = String::new();
        for v in self.source.iter() {
            out += &v.statement(phase,flags)[..];
        }
        out
    }

    pub(crate) fn serialise(&self, gpuspec: &GPUSpec, phase: Phase) -> String {
        let flags = self.get_flags();
        format!("{}\n\nvoid main() {{\n{}\n}}",
            self.declare(gpuspec,phase,&flags),
            self.statement(phase,&flags))
    }

    pub(crate) fn register(&self, builder: &mut ProgramBuilder, flags: &HashSet<String>) -> Result<(),Message> {
        for v in self.source.iter() {
            v.register(builder,flags)?;
        }
        Ok(())
    }
}
