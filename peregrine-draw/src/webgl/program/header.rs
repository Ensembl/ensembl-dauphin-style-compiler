use super::source::Source;
use super::program::Program;
use crate::util::message::Message;
use web_sys::WebGlRenderingContext;

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

    fn build(&mut self, context: &WebGlRenderingContext, program: &mut Program) -> Result<(),Message> {
        program.set_method(self.method);
        Ok(())
    }
}
