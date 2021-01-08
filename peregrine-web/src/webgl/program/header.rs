use super::source::{ Source, Runtime };

pub(crate) struct RuntimeHeader {

}

impl Runtime for RuntimeHeader {
    
}

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
    fn to_binary(&self) -> Box<dyn Runtime> {
        Box::new(RuntimeHeader {})
    }
}

