use std::fmt;
use peregrine_toolkit::serde::de_seq_next;
use serde::{Deserialize, Deserializer, Serializer};
use serde::de::{SeqAccess, Visitor};
use super::backoff::Backoff;
use super::channel::{ Channel, PacketPriority };
use crate::index::stickauthority::Authority;
use super::request::{RequestType};
use super::manager::RequestManager;
use crate::util::message::DataMessage;

#[derive(Clone)]
pub(super) struct AuthorityCommandRequest {}

impl AuthorityCommandRequest {
    fn new() -> AuthorityCommandRequest {
        AuthorityCommandRequest {}
    }

    async fn execute(self, channel: &Channel, manager: &RequestManager) -> Result<Authority,DataMessage> {
        let mut backoff = Backoff::new(manager,channel,&PacketPriority::RealTime);
        let response = backoff.backoff(RequestType::new_authority(self.clone()), |v| {
            v.into_authority()
        }).await?;
        Ok(Authority::new(&response.channel,&response.startup_name,&response.lookup_name,&response.jump_name))
    }
}

impl serde::Serialize for AuthorityCommandRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_none()
    }
}

pub struct AuthorityCommandResponse {
    channel: Channel,
    startup_name: String,
    lookup_name: String,
    jump_name: String
}

struct AuthorityVisitor;

impl<'de> Visitor<'de> for AuthorityVisitor {
    type Value = AuthorityCommandResponse;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f,"an authority") }

    fn visit_seq<S>(self, mut seq: S) -> Result<AuthorityCommandResponse,S::Error> where S: SeqAccess<'de> {
        let channel = de_seq_next(&mut seq)?;
        let startup_name = de_seq_next(&mut seq)?;
        let lookup_name = de_seq_next(&mut seq)?;
        let jump_name = de_seq_next(&mut seq)?;
        Ok(AuthorityCommandResponse { channel, startup_name, lookup_name, jump_name })
    }
}

impl<'de> Deserialize<'de> for AuthorityCommandResponse {
    fn deserialize<D>(deserializer: D) -> Result<AuthorityCommandResponse, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_seq(AuthorityVisitor)
    }
}

pub(super) async fn do_stick_authority(mut manager: RequestManager, channel: Channel) -> Result<Authority,DataMessage> {
    let req = AuthorityCommandRequest::new();
    req.execute(&channel,&mut manager).await
}
