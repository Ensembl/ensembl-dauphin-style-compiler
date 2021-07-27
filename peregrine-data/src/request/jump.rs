use std::any::Any;
use dauphin_interp::util::cbor::cbor_entry;
use serde_cbor::Value as CborValue;
use crate::util::cbor::{ cbor_string, cbor_map, cbor_int };
use super::backoff::Backoff;
use super::channel::{ Channel, PacketPriority };
use super::failure::GeneralFailure;
use super::request::{ RequestType, ResponseType, ResponseBuilderType };
use super::manager::RequestManager;
use crate::util::message::DataMessage;

#[derive(Clone)]
pub(crate) struct JumpCommandRequest {
    location: String
}

impl JumpCommandRequest {
    pub(crate) fn new(location: &str) -> JumpCommandRequest {
        JumpCommandRequest {
            location: location.to_string()
        }
    }

    pub(crate) async fn execute(self, channel: &Channel, manager: &mut RequestManager) -> anyhow::Result<Option<(String,u64,u64)>> {
        let mut backoff = Backoff::new();
        let r = backoff.backoff::<JumpCommandResponse,_,_>(manager,self.clone(),channel,PacketPriority::RealTime, |_| None).await??;
        Ok(r.0.map(|r| (r.stick.to_string(),r.start,r.end)))
    }
}

impl RequestType for JumpCommandRequest {
    fn type_index(&self) -> u8 { 5 }
    fn serialize(&self) -> Result<CborValue,DataMessage> {
        Ok(CborValue::Array(vec![CborValue::Text(self.location.to_string())]))
    }
    fn to_failure(&self) -> Box<dyn ResponseType> {
        Box::new(GeneralFailure::new("getting jump location"))
    }
}

struct JumpLocation {
    stick: String,
    start: u64,
    end: u64
}

struct JumpCommandResponse(Option<JumpLocation>);

impl ResponseType for JumpCommandResponse {
    fn as_any(&self) -> &dyn Any { self }
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
}

pub struct JumpResponseBuilderType();

impl ResponseBuilderType for JumpResponseBuilderType {
    fn deserialize(&self, value: &CborValue) -> anyhow::Result<Box<dyn ResponseType>> {
        let loc = if cbor_entry(value,"no")?.is_none() {
            let values = cbor_map(value,&["stick","left","right"])?;
            let stick = cbor_string(&values[0])?; 
            let start = cbor_int(&values[1],None)? as u64;
            let end = cbor_int(&values[2],None)? as u64;
            Some(JumpLocation { stick, start, end })
        } else {
            None
        };
        Ok(Box::new(JumpCommandResponse(loc)))
    }
}

pub async fn issue_jump_request(mut manager: RequestManager, channel: Channel, location: &str) -> anyhow::Result<Option<(String,u64,u64)>> {
    let req = JumpCommandRequest::new(&location);
    req.execute(&channel,&mut manager).await
}
