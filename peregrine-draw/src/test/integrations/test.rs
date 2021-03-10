use crate::PgDauphinIntegration;
use dauphin_interp::{ PayloadFactory, Stream, StreamConnector, Dauphin };
use dauphin_interp::runtime::Payload;
use serde_cbor::{ self, Value as CborValue };
use peregrine_data::{ PgConsole };
use std::rc::Rc;

#[derive(Clone)]
pub struct TestStreamConnector(Rc<dyn PgConsole>);

impl StreamConnector for TestStreamConnector {
    fn notice(&self, msg: &str) -> anyhow::Result<()> {
        self.0.error(msg);
        Ok(())
    }

    fn warn(&self, msg: &str) -> anyhow::Result<()> {
        self.0.error(msg);
        Ok(())
    }

    fn error(&self, msg: &str) -> anyhow::Result<()> {
        self.0.error(msg);
        Ok(())
    }
}

pub struct TestStreamFactory {
    console: Rc<dyn PgConsole>
}

impl TestStreamFactory {
    pub fn new(console: &Rc<dyn PgConsole>) -> TestStreamFactory {
        TestStreamFactory {
            console: console.clone()
        }
    }
}

impl PayloadFactory for TestStreamFactory {
    fn make_payload(&self) -> Box<dyn Payload> {
        Box::new(Stream::new(Box::new(TestStreamConnector(self.console.clone())),false,true))
    }
}

pub struct TestDauphinIntegration {
    console: Rc<dyn PgConsole>
}

impl TestDauphinIntegration {
    pub fn new(console: &Rc<dyn PgConsole>) -> TestDauphinIntegration {
        TestDauphinIntegration {
            console: console.clone()
        }
    }
}

impl PgDauphinIntegration for TestDauphinIntegration {
    fn add_payloads(&self, dauphin: &mut Dauphin) {
        dauphin.add_payload_factory("std","stream",Box::new(TestStreamFactory::new(&self.console)));
    }
}
