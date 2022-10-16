use std::fmt;
use serde::{Deserialize, Deserializer, de::{Visitor, IgnoredAny}};
use crate::{index::stickauthority::Authority, request::core::response::MiniResponseVariety};

pub struct AuthorityRes {}

impl AuthorityRes {
    pub fn build(&self) -> Authority {
        Authority::new()
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
        while let Some(IgnoredAny) = seq.next_element()? { /* ignore */ }
        Ok(AuthorityRes {})
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
