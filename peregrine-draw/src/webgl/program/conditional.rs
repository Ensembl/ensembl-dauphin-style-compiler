use std::collections::HashSet;

use crate::Message;
use crate::webgl::{GPUSpec, ProgramBuilder, SourceInstrs, Statement};

use super::source::Source;
use super::super::{ Phase };

#[derive(Clone)]
pub(crate) struct SetFlag(String);

impl SetFlag {
    pub fn new(flag:&str) -> Box<dyn Source> { Box::new(SetFlag(flag.to_string())) }
}

impl Source for SetFlag {
    fn cloned(&self) -> Box<dyn Source> { Box::new(self.clone()) }
    fn set_flags(&self, flags: &mut HashSet<String>) {
        flags.insert(self.0.clone());
    }
}

#[derive(Clone)]
pub(crate) struct Conditional {
    flag: String,
    source: SourceInstrs
}

impl Conditional {
    pub fn new(flag: &str, subs: Vec<Box<dyn Source>>) -> Box<Conditional> {
        Box::new(Conditional {
            flag: flag.to_string(),
            source: SourceInstrs::new(subs)
        })
    }
}

impl Source for Conditional {
    fn cloned(&self) -> Box<dyn Source> { Box::new(self.clone()) }

    fn declare(&self, spec: &GPUSpec, phase: Phase, flags: &HashSet<String>) -> String {
        if flags.contains(&self.flag) {
            self.source.declare(spec,phase,flags)
        } else {
            String::new()
        }
    }

    fn statement(&self, phase: Phase, flags: &HashSet<String>) -> String {
        if flags.contains(&self.flag) {
            self.source.statement(phase,flags)
        } else {
            String::new()
        }
    }

    fn register(&self, builder: &mut ProgramBuilder, flags: &HashSet<String>) -> Result<(),Message> { 
        if flags.contains(&self.flag) {
            self.source.register(builder,flags)?;
        }
        Ok(())
    }

    fn set_flags(&self, flags: &mut HashSet<String>) {
        if flags.contains(&self.flag) {
            self.set_flags(flags);
        }
    }
}
