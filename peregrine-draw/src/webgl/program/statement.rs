use std::collections::HashSet;

use crate::webgl::GPUSpec;

use super::source::Source;
use super::super::{ Phase };

#[derive(Clone)]
pub(crate) struct Statement {
    statement: String,
    phase: Phase
}

impl Statement {
    pub fn new_vertex(statement: &str) -> Box<Statement> {
        Box::new(Statement {
            statement: statement.to_string(),
            phase: Phase::Vertex
        })
    }

    pub fn new_fragment(statement: &str) -> Box<Statement> {
        Box::new(Statement {
            statement: statement.to_string(),
            phase: Phase::Fragment
        })
    }
}

impl Source for Statement {
    fn cloned(&self) -> Box<dyn Source> { Box::new(self.clone()) }

    fn statement(&self,  phase: Phase, _flags: &HashSet<String>) -> String {
        if phase != self.phase { return String::new(); }
        format!("{};\n",self.statement)
    }  
}

#[derive(Clone)]
pub(crate) struct Declaration {
    statement: String,
    phase: Phase
}

impl Declaration {
    pub fn new_vertex(statement: &str) -> Box<Declaration> {
        Box::new(Declaration {
            statement: statement.to_string(),
            phase: Phase::Vertex
        })
    }

    pub fn new_fragment(statement: &str) -> Box<Declaration> {
        Box::new(Declaration {
            statement: statement.to_string(),
            phase: Phase::Fragment
        })
    }
}

impl Source for Declaration {
    fn cloned(&self) -> Box<dyn Source> { Box::new(self.clone()) }

    fn declare(&self, gpu_spec: &GPUSpec, phase: Phase, _flags: &HashSet<String>) -> String {
        if phase != self.phase { return String::new(); }
        format!("{}\n",self.statement)
    }  
}
