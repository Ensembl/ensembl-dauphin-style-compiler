use std::fmt;
use serde::{de::Visitor, Deserialize, Deserializer};

use crate::request::core::miniresponse::MiniResponseVariety;

pub struct ExpandRes;

struct ExpandResVisitor;

impl<'de> Visitor<'de> for ExpandResVisitor {
    type Value = ExpandRes;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an ExpandRes")
    }

    fn visit_seq<A>(self, _: A) -> Result<Self::Value, A::Error>
            where A: serde::de::SeqAccess<'de>, {
        Ok(ExpandRes)
    }
}

impl<'de> Deserialize<'de> for ExpandRes {
    fn deserialize<D>(deserializer: D) -> Result<ExpandRes, D::Error>
            where D: Deserializer<'de> {
        deserializer.deserialize_seq(ExpandResVisitor)
    }
}

impl MiniResponseVariety for ExpandRes {
    fn description(&self) -> &str { "expand" }
}
