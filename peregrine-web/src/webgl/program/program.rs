use super::super::Source;

pub(crate) struct Program {
    source: Vec<Box<dyn Source>>
}

impl Program {
    pub fn new(source: Vec<Box<dyn Source>>) -> Program {
        Program {
            source
        }
    }

    pub fn merge(&mut self, mut other: Program) {
        self.source.extend(other.source.drain(..));
    }
}
