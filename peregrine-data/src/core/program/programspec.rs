use std::collections::HashMap;

use peregrine_toolkit::{error::Error, eachorevery::eoestruct::StructBuilt};

use super::packedprogramspec::PackedProgramSpec;

#[derive(Clone,Debug)]
pub(crate) struct ProgramSetting {
    name: String,
    default: StructBuilt
}

impl ProgramSetting {
    pub fn new(name: &str, default: StructBuilt) -> ProgramSetting {
        ProgramSetting {
            name: name.to_string(),
            default
        }
    }
}

#[derive(Clone,Debug)]
pub(crate) struct ProgramModel {
    name: String,
    in_bundle_name: String,
    set: String,
    version: usize,
    settings: HashMap<String,ProgramSetting>
}

impl ProgramModel {
    pub fn new(set: &str, name: &str, version: usize, in_bundle_name: &str) -> ProgramModel {
        ProgramModel {
            name: name.to_string(), 
            in_bundle_name: in_bundle_name.to_string(),
            set: set.to_string(),
            version,
            settings: HashMap::new()
        }
    }

    pub fn add_setting(&mut self, name: &str, setting: ProgramSetting) {
        self.settings.insert(name.to_string(),setting);
    }
}

pub(crate) enum ProgramSpec {
    Umpacked(Vec<ProgramModel>),
    Packed(PackedProgramSpec)
}

impl ProgramSpec {
    pub(crate) fn to_program_models(&mut self) -> Result<Vec<ProgramModel>,Error> {
        match self {
            ProgramSpec::Umpacked(m) => Ok(m.clone()),
            ProgramSpec::Packed(m) => m.to_program_models()
        }
    }
}
