use std::collections::HashSet;
use peregrine_toolkit::error::Error;
use super::source::Source;
use super::program::{ ProgramBuilder };

#[derive(Clone)]
pub(crate) struct Header {
    method: u32
}

impl Header {
    pub fn new(method: u32) -> Box<Header> {
        Box::new(Header {
            method
        })
    }
}

impl Source for Header {
    fn cloned(&self) -> Box<dyn Source> { Box::new(self.clone()) }

    fn register(&self, builder: &mut ProgramBuilder, _flags: &HashSet<String>) -> Result<(),Error> {
        builder.set_method(self.method);
        Ok(())
    }
}
