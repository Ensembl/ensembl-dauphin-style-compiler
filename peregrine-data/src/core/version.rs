use serde::{Serialize, ser::SerializeMap};

const BE_VERSION: u32 = 16;

#[derive(Clone)]
pub struct VersionMetadata {
    be_version: u32
}

impl VersionMetadata {
    pub fn new() -> VersionMetadata {
        VersionMetadata {
            be_version: BE_VERSION
        }
    }

    pub fn backend_version(&self) -> u32 { self.be_version }
}

impl Serialize for VersionMetadata {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("egs",&self.be_version)?;
        map.end()
    }
}
