use std::collections::HashMap;

use peregrine_toolkit::{error::Error, eachorevery::eoestruct::StructBuilt};

use crate::{shapeload::programname::ProgramName, BackendNamespace};

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
    name: ProgramName,
    in_bundle_name: String,
    settings: HashMap<String,ProgramSetting>
}

impl ProgramModel {
    pub fn new(name: &ProgramName, in_bundle_name: &str) -> ProgramModel {
        ProgramModel {
            name: name.clone(), 
            in_bundle_name: in_bundle_name.to_string(),
            settings: HashMap::new()
        }
    }

    pub fn add_setting(&mut self, name: &str, setting: ProgramSetting) {
        self.settings.insert(name.to_string(),setting);
    }

    pub fn name(&self) -> &ProgramName { &self.name }
    pub fn in_bundle_name(&self) -> &str { &self.in_bundle_name }
}

pub(crate) enum ProgramSpec {
    Umpacked(Vec<ProgramModel>),
    Packed(PackedProgramSpec)
}

impl ProgramSpec {
    pub(crate) fn to_program_models(&self, backend_namespace: &BackendNamespace) -> Result<Vec<ProgramModel>,Error> {
        match self {
            ProgramSpec::Umpacked(m) => Ok(m.clone()),
            ProgramSpec::Packed(m) => m.to_program_models(backend_namespace)
        }
    }
}
