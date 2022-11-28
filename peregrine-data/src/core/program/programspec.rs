use std::{collections::{HashMap, BTreeMap}, sync::Arc, fmt};
use peregrine_toolkit::{error::Error, eachorevery::eoestruct::{StructValue}};
use serde::{Deserializer, de::{MapAccess, Visitor}, Deserialize};
use crate::{shapeload::programname::ProgramName };
use super::packedprogramspec::PackedProgramSpec;

#[derive(Clone)]
pub struct ProgramSetting {
    name: String,
    default: StructValue
}

impl ProgramSetting {
    pub fn new(name: &str, default: StructValue) -> ProgramSetting {
        ProgramSetting {
            name: name.to_string(),
            default
        }
    }
}

#[derive(serde_derive::Deserialize)]
pub struct NamelessProgramSetting {
    default: StructValue
}

struct SettingsVisitor;

impl<'de> Visitor<'de> for SettingsVisitor {
    type Value = HashMap<String,ProgramSetting>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a A settings hash")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where M: MapAccess<'de> {
        let mut out = HashMap::new();
        while let Some(name) = access.next_key::<String>()? {
            let value : NamelessProgramSetting = access.next_value()?;
            out.insert(name.clone(),ProgramSetting {
                name,
                default: value.default
            });
        }
        Ok(out)
    }
}

fn settings_ds<'de,D>(deserializer: D) -> Result<HashMap<String,ProgramSetting>,D::Error> where D: Deserializer<'de> {
    deserializer.deserialize_seq(SettingsVisitor)
}

#[derive(serde_derive::Deserialize)]
pub(crate) struct ProgramModelBuilder {
    name: ProgramName,
    in_bundle_name: String,
    #[serde(deserialize_with = "settings_ds",default)]
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

    pub fn apply_defaults(&self, settings: &mut BTreeMap<String,StructValue>) {
        for (_key,value) in self.0.settings.iter() {
            let name = &value.name;
            if !settings.contains_key(name) {
                settings.insert(name.to_string(),value.default.clone());
            }
        }
    }
}

pub(crate) enum ProgramSpec {
    Unpacked(Vec<ProgramModel>),
    Packed(PackedProgramSpec)
}

impl ProgramSpec {
    pub(crate) fn to_program_models(&self) -> Result<Vec<ProgramModel>,Error> {
        match self {
            ProgramSpec::Unpacked(m) => Ok(m.clone()),
            ProgramSpec::Packed(m) => m.to_program_models()
        }
    }
}

impl<'de> Deserialize<'de> for ProgramModel {
    fn deserialize<D>(deserializer: D) -> Result<ProgramModel, D::Error>
            where D: Deserializer<'de> {
        Ok(ProgramModel::new(ProgramModelBuilder::deserialize(deserializer)?))
    }
}

impl<'de> Deserialize<'de> for ProgramSpec {
    fn deserialize<D>(deserializer: D) -> Result<ProgramSpec, D::Error>
            where D: Deserializer<'de> {
        Ok(ProgramSpec::Unpacked(<Vec<ProgramModel>>::deserialize(deserializer)?))
    }
}
