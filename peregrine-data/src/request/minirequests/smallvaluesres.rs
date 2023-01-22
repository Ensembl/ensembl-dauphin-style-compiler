use std::{fmt, collections::HashMap};
use peregrine_toolkit::serdetools::st_field;
use serde::{Deserialize, de::Visitor, Deserializer};
use crate::request::core::miniresponse::MiniResponseVariety;

pub struct SmallValuesRes(HashMap<String,String>);

impl SmallValuesRes {
    pub fn empty() -> SmallValuesRes { SmallValuesRes(HashMap::new()) }

    pub fn small_values(&self) -> &HashMap<String,String> { &self.0 }
}

impl MiniResponseVariety for SmallValuesRes {
    fn description(&self) -> &str { "small-value" }
}

struct SmallValuesResResVisitor;

impl<'de> Visitor<'de> for SmallValuesResResVisitor {
    type Value = SmallValuesRes;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an SmallValuesRes")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where A: serde::de::SeqAccess<'de> {
        let reason = st_field("values",seq.next_element()?)?;
        Ok(SmallValuesRes(reason))
    }
}

impl<'de> Deserialize<'de> for SmallValuesRes {
    fn deserialize<D>(deserializer: D) -> Result<SmallValuesRes, D::Error>
            where D: Deserializer<'de> {
        deserializer.deserialize_seq(SmallValuesResResVisitor)
    }
}
