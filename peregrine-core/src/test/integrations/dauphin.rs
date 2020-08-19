use dauphin_interp::Dauphin;
use crate::PgDauphinIntegration;
use serde_cbor::{ self, Value as CborValue };
use dauphin_interp::runtime::{ Payload, PayloadFactory };
use dauphin_interp::{ StreamConnector, Stream };
use super::console::TestConsole;

pub struct TestStreamConnector(TestConsole);

impl StreamConnector for TestStreamConnector {
    fn notice(&self, msg: &str) -> anyhow::Result<()> {
        self.0.message(msg);
        Ok(())
    }

    fn warn(&self, msg: &str) -> anyhow::Result<()> {
        self.0.message(msg);
        Ok(())
    }

    fn error(&self, msg: &str) -> anyhow::Result<()> {
        self.0.message(msg);
        Ok(())
    }
}

pub struct TestStreamFactory {
    console: TestConsole
}

impl TestStreamFactory {
    pub fn new(console: &TestConsole) -> TestStreamFactory {
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
    console: TestConsole
}

impl TestDauphinIntegration {
    pub fn new(console: &TestConsole) -> TestDauphinIntegration {
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

pub fn test_program() -> CborValue {
    let bytes = include_bytes!("../test.dpb");
    serde_cbor::from_slice(bytes).expect("bad test program")
}