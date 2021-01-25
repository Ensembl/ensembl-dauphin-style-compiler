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

    fn statement(&self,  phase: Phase) -> String {
        if phase != self.phase { return String::new(); }
        format!("{};\n",self.statement)
    }  
}
