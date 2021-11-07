use std::fmt;

use serde::de::{SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serializer};
use super::backoff::Backoff;
use super::channel::{ PacketPriority };
use super::request::{RequestType};
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
        let _r = backoff.backoff(RequestType::new_program(self.clone()), |v| {
            v.into_program()
        }).await?;
        Ok(())
    }
}

impl serde::Serialize for ProgramCommandRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        self.program_name.serialize(serializer)
    }
}

pub struct ProgramCommandResponse {}

pub(super) async fn do_load_program(manager: &RequestManager, program_name: ProgramName) -> Result<(),DataMessage> {
    let req = ProgramCommandRequest::new(&program_name);
    req.execute(manager).await?;
    Ok(())
}

struct ProgramVisitor;

impl<'de> Visitor<'de> for ProgramVisitor {
    type Value = ProgramCommandResponse;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f,"a program response") }

    fn visit_seq<S>(self, _seq: S) -> Result<ProgramCommandResponse,S::Error> where S: SeqAccess<'de> {
        Ok(ProgramCommandResponse{})
    }
}

impl<'de> Deserialize<'de> for ProgramCommandResponse {
    fn deserialize<D>(deserializer: D) -> Result<ProgramCommandResponse, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_seq(ProgramVisitor)
    }
}
