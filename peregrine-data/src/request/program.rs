use crate::Channel;
use std::any::Any;
use serde_cbor::Value as CborValue;
use super::backoff::Backoff;
use super::channel::{ PacketPriority };
use super::failure::GeneralFailure;
use super::request::{ RequestType, ResponseType, ResponseBuilderType };
use super::manager::RequestManager;
use crate::util::message::DataMessage;
use crate::lane::programname::ProgramName;

#[derive(Clone)]
struct ProgramCommandRequest {
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
        backoff.backoff::<ProgramCommandResponse,_>(self.clone()).await??;
        Ok(())
    }
}

impl RequestType for ProgramCommandRequest {
    fn type_index(&self) -> u8 { 1 }
    fn serialize(&self, _channel: &Channel) -> Result<CborValue,DataMessage> {
        self.program_name.serialize()
    }
    fn to_failure(&self) -> Box<dyn ResponseType> {
        Box::new(GeneralFailure::new("program loading failed"))
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
