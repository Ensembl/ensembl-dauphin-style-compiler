use std::{collections::HashMap, fmt};
use peregrine_toolkit::serde::de_seq_next;
use serde::{Deserialize, Deserializer, de::{SeqAccess, Visitor}};
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

struct BundleVisitor;

impl<'de> Visitor<'de> for BundleVisitor {
    type Value = SuppliedBundle;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f,"a channel") }

    fn visit_seq<S>(self, mut seq: S) -> Result<SuppliedBundle,S::Error> where S: SeqAccess<'de> {
        let bundle_name = de_seq_next(&mut seq)?;
        let program = de_seq_next(&mut seq)?;
        let names = de_seq_next(&mut seq)?;
        Ok(SuppliedBundle { bundle_name, program, names })
    }
}

impl<'de> Deserialize<'de> for SuppliedBundle {
    fn deserialize<D>(deserializer: D) -> Result<SuppliedBundle, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_seq(BundleVisitor)
    }
}
