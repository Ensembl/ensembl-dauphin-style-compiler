use std::fmt;
use peregrine_toolkit::{serdetools::{st_field, ByteData}};
use serde::{de::{Visitor, MapAccess, IgnoredAny}, Deserialize, Deserializer};
use super::{packedprogramspec::PackedProgramSpec, programspec::{ProgramSpec}};

pub struct SuppliedBundle {
    bundle_name: String,
    code: Vec<u8>,
    specs: ProgramSpec
}

impl SuppliedBundle {
    pub(crate) fn bundle_name(&self) -> &str { &self.bundle_name }
    pub(crate) fn code(&self) -> &[u8] { &self.code }
    pub(crate) fn specs(&self) -> &ProgramSpec { &self.specs }
}

pub struct PackedSuppliedBundle(pub SuppliedBundle);

struct PackedProgramBundleVisitor;

impl<'de> Visitor<'de> for PackedProgramBundleVisitor {
    type Value = PackedSuppliedBundle;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a SuppliedBundle")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where M: MapAccess<'de> {
        let mut bundle_name = None;
        let mut code : Option<ByteData> = None;
        let mut specs = None;
        while let Some(key) = access.next_key()? {
            match key {
                "bundle_name" => { bundle_name = Some(access.next_value()?); },
                "code" => { code = Some(access.next_value()?); },
                "specs" => {
                    let packed : PackedProgramSpec = access.next_value()?;
                    specs = Some(ProgramSpec::Packed(packed));
                },
                _ => { let _ : IgnoredAny = access.next_value()?; }
            }
        }
        let code = st_field("code",code)?;
        let bundle_name = st_field("bundle_name",bundle_name)?;
        let specs = st_field("specs",specs)?;
        Ok(PackedSuppliedBundle(SuppliedBundle {
            bundle_name, code: code.data, specs
        }))
    }
}

impl<'de> Deserialize<'de> for PackedSuppliedBundle {
    fn deserialize<D>(deserializer: D) -> Result<PackedSuppliedBundle, D::Error>
            where D: Deserializer<'de> {
        deserializer.deserialize_map(PackedProgramBundleVisitor)
    }
}

#[derive(serde_derive::Deserialize)]
pub struct UnpackedSuppliedBundle {
    bundle_name: String,
    code: ByteData,
    specs: ProgramSpec
}

impl UnpackedSuppliedBundle {
    pub fn to_supplied_bundle(self) -> SuppliedBundle {
        SuppliedBundle {
            bundle_name: self.bundle_name,
            code: self.code.data,
            specs: self.specs
        }
    }
}
