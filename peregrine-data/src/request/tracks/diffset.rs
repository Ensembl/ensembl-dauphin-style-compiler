use std::fmt;
use serde::{Deserialize, Deserializer, de::Visitor};

struct DiffSetVisitor;

impl<'de> Visitor<'de> for DiffSetVisitor {
    type Value = DiffSet;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a DiffSet")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where A: serde::de::SeqAccess<'de> {
        let mut out = vec![];
        let mut prev_value = 0;
        while let Some(value) = seq.next_element::<usize>()? {
            prev_value += value;
            out.push(prev_value);
        }
        Ok(DiffSet(out))
    }
}

#[derive(Clone,Debug)]
pub(super) struct DiffSet(pub(super) Vec<usize>);

impl<'de> Deserialize<'de> for DiffSet {
    fn deserialize<D>(deserializer: D) -> Result<DiffSet, D::Error>
            where D: Deserializer<'de> {
        deserializer.deserialize_map(DiffSetVisitor)
    }
}
