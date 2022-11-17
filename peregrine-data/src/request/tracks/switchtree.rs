use std::fmt;
use serde::{Deserialize, Deserializer, de::Visitor};


#[derive(serde_derive::Deserialize)]
struct SwitchTreeElement {
    prefix: i32,
    suffix: Vec<String>
}

struct SwitchTreeVisitor;

impl<'de> Visitor<'de> for SwitchTreeVisitor {
    type Value = SwitchTree;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a SwitchTree")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where A: serde::de::SeqAccess<'de> {
        let mut out = vec![];
        let mut prev_value = vec![];
        let mut prefix : i32 = 0;
        while let Some(mut el) = seq.next_element::<SwitchTreeElement>()? {
            prefix += el.prefix as i32;
            if prefix > prev_value.len() as i32 || prefix < 0 {
                return Err(serde::de::Error::custom("bad prefix"));
            }
            let mut value : Vec<String> = prev_value[0..prefix as usize].iter().cloned().collect();
            value.append(&mut el.suffix);
            out.push(value.clone());
            prev_value = value;
        }
        Ok(SwitchTree(out))
    }
}

#[derive(Debug)]
pub(super) struct SwitchTree(pub(super) Vec<Vec<String>>);

impl<'de> Deserialize<'de> for SwitchTree {
    fn deserialize<D>(deserializer: D) -> Result<SwitchTree, D::Error>
            where D: Deserializer<'de> {
        deserializer.deserialize_map(SwitchTreeVisitor)
    }
}
