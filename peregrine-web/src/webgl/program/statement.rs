use super::source::Source;
use super::super::{ GLArity, GPUSpec, Precision, Phase };

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
    fn statement(&self,  phase: Phase) -> String {
        if phase != self.phase { return String::new(); }
        format!("{};\n",self.statement)
    }  
}
