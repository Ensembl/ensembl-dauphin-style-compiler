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
        while let Some(value) = seq.next_element::<i64>()? {
            prev_value += value;
            out.push(prev_value as usize);
        }
        Ok(DiffSet(out))
    }
}

#[derive(Clone,Debug)]
pub struct DiffSet(pub Vec<usize>);

impl DiffSet {
    pub fn iter(&self) -> impl Iterator<Item=&usize> {
        self.0.iter()
    }

    pub fn len(&self) -> usize { self.0.len() }
}

impl<'de> Deserialize<'de> for DiffSet {
    fn deserialize<D>(deserializer: D) -> Result<DiffSet, D::Error>
            where D: Deserializer<'de> {
        deserializer.deserialize_seq(DiffSetVisitor)
    }
}
