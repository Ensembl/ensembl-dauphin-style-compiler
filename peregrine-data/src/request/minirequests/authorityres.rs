use std::fmt;
use peregrine_toolkit::{serdetools::st_field };
use serde::{Deserialize, Deserializer, de::{Visitor, IgnoredAny}};
use crate::{index::stickauthority::Authority, BackendNamespace, request::core::response::MiniResponseVariety};

pub struct AuthorityRes {
    channel: BackendNamespace,
    startup_name: String
}

impl AuthorityRes {
    pub fn build(&self) -> Authority {
        Authority::new(&self.channel,&self.startup_name)
    }
}

struct AuthorityVisitor;

impl<'de> Visitor<'de> for AuthorityVisitor {
    type Value = AuthorityRes;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an AuthorityRes")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where A: serde::de::SeqAccess<'de>, {
        let channel = st_field("channel",seq.next_element()?)?;
        let startup_name = st_field("startup_name",seq.next_element()?)?;
        while let Some(IgnoredAny) = st_field("tail",seq.next_element()?)? { /* Ignore rest */ }
        Ok(AuthorityRes { channel, startup_name })
    }
}

impl<'de> Deserialize<'de> for AuthorityRes {
    fn deserialize<D>(deserializer: D) -> Result<AuthorityRes, D::Error>
            where D: Deserializer<'de> {
        deserializer.deserialize_seq(AuthorityVisitor)
    }
}

impl MiniResponseVariety for AuthorityRes {
    fn description(&self) -> &str { "authority" }
}
