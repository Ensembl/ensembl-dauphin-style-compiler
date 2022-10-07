use std::{collections::HashMap, fmt};
use peregrine_toolkit::{serdetools::st_field};
use serde::{de::Visitor, Deserialize, Deserializer};
use serde_cbor::Value as CborValue;

pub struct SuppliedBundle {
    bundle_name: String,
    program: CborValue,
    names: HashMap<String,String> // in-channel name -> in-bundle name
}

impl SuppliedBundle {
    pub(crate) fn bundle_name(&self) -> &str { &self.bundle_name }
    pub(crate) fn program(&self) -> &CborValue { &self.program }
    pub(crate) fn name_map(&self) -> impl Iterator<Item=(&str,&str)> {
        self.names.iter().map(|(x,y)| (x as &str,y as &str))
    }
}

struct ProgramBundleVisitor;

impl<'de> Visitor<'de> for ProgramBundleVisitor {
    type Value = SuppliedBundle;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a SuppliedBundle")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where A: serde::de::SeqAccess<'de> {
        let bundle_name = st_field("bundle_name",seq.next_element()?)?;
        let program = st_field("program",seq.next_element()?)?;
        let names = st_field("names",seq.next_element()?)?;
        Ok(SuppliedBundle { bundle_name, program, names })
    }
}

impl<'de> Deserialize<'de> for SuppliedBundle {
    fn deserialize<D>(deserializer: D) -> Result<SuppliedBundle, D::Error>
            where D: Deserializer<'de> {
        deserializer.deserialize_seq(ProgramBundleVisitor)
    }
}
