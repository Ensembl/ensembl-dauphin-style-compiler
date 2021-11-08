use std::fmt;
use peregrine_toolkit::serde::de_seq_next;
use serde::{Deserialize, Deserializer, de::{SeqAccess, Visitor}};
use crate::{core::channel::Channel, index::stickauthority::Authority};

pub struct AuthorityCommandResponse {
    channel: Channel,
    startup_name: String,
    lookup_name: String,
    jump_name: String
}

impl AuthorityCommandResponse {
    pub fn build(&self) -> Authority {
        Authority::new(&self.channel,&self.startup_name,&self.lookup_name,&self.jump_name)
    }
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
