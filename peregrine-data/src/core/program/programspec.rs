use std::{collections::{HashMap, BTreeMap}, sync::Arc};

use peregrine_toolkit::{error::Error, eachorevery::eoestruct::StructBuilt};

use crate::{shapeload::programname::ProgramName };

use super::packedprogramspec::PackedProgramSpec;

#[derive(Clone)]
pub struct ProgramSetting {
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

pub(crate) struct ProgramModelBuilder {
    name: ProgramName,
    in_bundle_name: String,
    settings: HashMap<String,ProgramSetting>
}

impl ProgramModelBuilder {
    pub fn new(name: &ProgramName, in_bundle_name: &str) -> ProgramModelBuilder {
        ProgramModelBuilder {
            name: name.clone(), 
            in_bundle_name: in_bundle_name.to_string(),
            settings: HashMap::new()
        }
    }

    pub fn add_setting(&mut self, name: &str, setting: ProgramSetting) {
        self.settings.insert(name.to_string(),setting);
    }
}

#[derive(Clone)]
pub struct ProgramModel(Arc<ProgramModelBuilder>);

impl ProgramModel {
    pub(crate) fn new(builder: ProgramModelBuilder) -> ProgramModel {
        ProgramModel(Arc::new(builder))
    }

    pub fn name(&self) -> &ProgramName { &self.0.name }

    pub fn in_bundle_name(&self) -> &str { &self.0.in_bundle_name }
    pub fn get_setting(&self, setting: &str) -> Option<&ProgramSetting> {
        self.0.settings.get(setting)
    }

    pub fn apply_defaults(&self, settings: &mut BTreeMap<String,StructBuilt>) {
        for (_key,value) in self.0.settings.iter() {
            let name = &value.name;
            if !settings.contains_key(name) {
                settings.insert(name.to_string(),value.default.clone());
            }
        }
    }
}

pub(crate) enum ProgramSpec {
    Umpacked(Vec<ProgramModel>),
    Packed(PackedProgramSpec)
}

impl ProgramSpec {
    pub(crate) fn to_program_models(&self) -> Result<Vec<ProgramModel>,Error> {
        match self {
            ProgramSpec::Umpacked(m) => Ok(m.clone()),
            ProgramSpec::Packed(m) => m.to_program_models()
        }
    }
}
