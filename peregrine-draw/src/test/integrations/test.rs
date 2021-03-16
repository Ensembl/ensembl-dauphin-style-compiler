use crate::PgDauphinIntegration;
use dauphin_interp::{ PayloadFactory, Stream, StreamConnector, Dauphin };
use dauphin_interp::runtime::Payload;
use serde_cbor::{ self, Value as CborValue };
use std::rc::Rc;

#[derive(Clone)]
pub struct TestStreamConnector(MessageSender);

impl StreamConnector for TestStreamConnector {
    fn notice(&self, msg: &str) -> anyhow::Result<()> {
        self.0.send(msg);
        Ok(())
    }

    fn warn(&self, msg: &str) -> anyhow::Result<()> {
        self.0.send(msg);
        Ok(())
    }

    fn error(&self, msg: &str) -> anyhow::Result<()> {
        self.0.send(msg);
        Ok(())
    }
}

pub struct TestStreamFactory {
    messages: MessageSender
}

impl TestStreamFactory {
    pub fn new(messages: &MessageSender) -> TestStreamFactory {
        TestStreamFactory {
            messages: messages.clone()
        }
    }
}

impl PayloadFactory for TestStreamFactory {
    fn make_payload(&self) -> Box<dyn Payload> {
        Box::new(Stream::new(Box::new(TestStreamConnector(self.messages.clone())),false,true))
    }
}

pub struct TestDauphinIntegration {
    messages: MessageSender
}

impl TestDauphinIntegration {
    pub fn new(messages: &MessageSender) -> TestDauphinIntegration {
        TestDauphinIntegration {
            messages: messages.clone()
        }
    }
}

impl PgDauphinIntegration for TestDauphinIntegration {
    fn add_payloads(&self, dauphin: &mut Dauphin) {
        dauphin.add_payload_factory("std","stream",Box::new(TestStreamFactory::new(&self.messages)));
    }
}
