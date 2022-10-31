use std::{collections::HashMap, fmt};
use peregrine_toolkit::{serdetools::{st_field, ByteData}};
use serde::{de::{Visitor, MapAccess, IgnoredAny}, Deserialize, Deserializer};

pub struct SuppliedBundle {
    bundle_name: String,
    program: Vec<u8>,
    names: HashMap<String,String> // in-channel name -> in-bundle name
}

impl SuppliedBundle {
    pub(crate) fn bundle_name(&self) -> &str { &self.bundle_name }
    pub(crate) fn program(&self) -> &[u8] { &self.program }
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

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where M: MapAccess<'de> {
        let mut bundle_name = None;
        let mut code : Option<ByteData> = None;
        let mut name_mapping = HashMap::new();
        while let Some(key) = access.next_key()? {
            match key {
                "bundle_name" => { bundle_name = Some(access.next_value()?); },
                "code" => { code = Some(access.next_value()?); },
                "name_mapping" => { name_mapping = access.next_value()?; },
                _ => { let _ : IgnoredAny = access.next_value()?; }
            }
        }
        let code = st_field("code",code)?;
        let bundle_name = st_field("bundle_name",bundle_name)?;
        Ok(SuppliedBundle {
            bundle_name, program: code.data, names: name_mapping
        })
    }
}

impl<'de> Deserialize<'de> for SuppliedBundle {
    fn deserialize<D>(deserializer: D) -> Result<SuppliedBundle, D::Error>
            where D: Deserializer<'de> {
        deserializer.deserialize_map(ProgramBundleVisitor)
    }
}
