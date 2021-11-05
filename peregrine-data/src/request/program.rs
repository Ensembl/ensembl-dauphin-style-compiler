use std::any::Any;
use serde::Serializer;
use serde_cbor::Value as CborValue;
use super::backoff::Backoff;
use super::channel::{ PacketPriority };
use super::request::{RequestType, ResponseBuilderType, ResponseType};
use super::manager::RequestManager;
use crate::util::message::DataMessage;
use crate::lane::programname::ProgramName;

#[derive(Clone)]
pub(super) struct ProgramCommandRequest {
    program_name: ProgramName
}

impl ProgramCommandRequest {
    fn new(program_name: &ProgramName) -> ProgramCommandRequest {
        ProgramCommandRequest {
            program_name: program_name.clone()
        }
    }

    async fn execute(self, manager: &RequestManager) -> Result<(),DataMessage> {
        let mut backoff = Backoff::new(manager,&self.program_name.0,&PacketPriority::RealTime);
        backoff.backoff_new::<ProgramCommandResponse>(RequestType::new_program(self.clone())).await??;
        Ok(())
    }
}

impl serde::Serialize for ProgramCommandRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        self.program_name.serialize(serializer)
    }
}

struct ProgramCommandResponse {}

impl ResponseType for ProgramCommandResponse {
    fn as_any(&self) -> &dyn Any { self }
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
}

pub struct ProgramResponseBuilderType();

impl ResponseBuilderType for ProgramResponseBuilderType {
    fn deserialize(&self, _value: &CborValue) -> anyhow::Result<Box<dyn ResponseType>> {
        Ok(Box::new(ProgramCommandResponse {}))
    }
}

pub(super) async fn do_load_program(manager: &RequestManager, program_name: ProgramName) -> Result<(),DataMessage> {
    let req = ProgramCommandRequest::new(&program_name);
    req.execute(manager).await?;
    Ok(())
}
