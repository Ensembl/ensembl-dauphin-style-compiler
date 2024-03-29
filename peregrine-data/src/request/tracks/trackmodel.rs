use std::{sync::Arc, collections::BTreeMap, mem};
use eachorevery::eoestruct::StructValue;
use peregrine_toolkit::{ error::Error };
use serde::{Deserialize, Deserializer, de::DeserializeSeed};
use crate::{Track, shapeload::programname::ProgramName, PgDauphin, switch::switches::SwitchesData, BackendNamespace };

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(serde_derive::Deserialize,PartialEq,Eq,Hash,PartialOrd,Ord)]
pub(crate) struct TrackMappingBuilder {
    settings: BTreeMap<String,Vec<String>>,
    values: BTreeMap<String,StructValue>,
}

impl TrackMappingBuilder {
    fn new() -> TrackMappingBuilder {
        TrackMappingBuilder {
            settings: BTreeMap::new(),
            values: BTreeMap::new()
        }
    }

    pub(crate) fn add_setting(&mut self, key: &str, path: &[String]) {
        self.settings.insert(key.to_string(),path.to_vec());
    }

    pub(crate) fn add_value(&mut self, key: &str, value: StructValue) {
        self.values.insert(key.to_string(),value);
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone,PartialEq,Eq,Hash,PartialOrd,Ord)]
pub struct TrackMapping(Arc<TrackMappingBuilder>);

impl TrackMapping {
    fn new(builder: TrackMappingBuilder) -> TrackMapping {
        TrackMapping(Arc::new(builder))
    }

    pub(crate) fn apply(&self, switches_data: &SwitchesData) -> (BTreeMap<String,StructValue>,BTreeMap<String,Vec<String>>) {
        let mut values = BTreeMap::new();
        let mut sources = BTreeMap::new();
        for (key,value) in &self.0.values {
            values.insert(key.to_string(),value.clone());
        }
        for (key,path) in &self.0.settings {
            let path_str = path.iter().map(|x| x.as_str()).collect::<Vec<_>>();
            sources.insert(key.to_string(),path.to_vec());
            values.insert(key.to_string(),switches_data.get_value(&path_str));
        }
        (values,sources)
    }

    pub(crate) fn get_switch(&self, setting: &str) -> Option<&[String]> {
        self.0.settings.get(setting).map(|x| x.as_slice())
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(serde_derive::Deserialize,PartialEq,Eq,Hash,PartialOrd,Ord)]
pub struct TrackModelBuilder {
    program: ProgramName,
    tags: String,
    triggers: Vec<Vec<String>>,
    #[serde(flatten)]
    mapping: TrackMappingBuilder,
    scale_start: u64,
    scale_end: u64,
    scale_step: u64
}

impl TrackModelBuilder {
    pub fn new(program: &ProgramName, scale_start: u64, scale_end: u64, scale_step: u64, tags: &str) -> TrackModelBuilder {
        TrackModelBuilder {
            program: program.clone(),
            tags: tags.to_string(),
            triggers: vec![],
            mapping: TrackMappingBuilder::new(),
            scale_start, scale_end, scale_step
        }
    }

    pub(crate) fn mapping_mut(&mut self) -> &mut TrackMappingBuilder { &mut self.mapping }
    pub fn add_trigger(&mut self, trigger: &[String]) { self.triggers.push(trigger.to_vec()) }
}

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone,PartialEq,Eq,Hash,PartialOrd,Ord)]
pub struct TrackModel {
    builder: Arc<TrackModelBuilder>,
    track_base: BackendNamespace,
    track_mapping: TrackMapping
}

impl TrackModel {
    pub fn new(mut builder: TrackModelBuilder, track_base: &BackendNamespace) -> TrackModel {
        let mapping = mem::replace(&mut builder.mapping,TrackMappingBuilder::new());
        TrackModel {
            track_mapping: TrackMapping::new(mapping),
            track_base: track_base.clone(),
            builder: Arc::new(builder)
        }
    }

    pub(crate) async fn to_track(&self, loader: &PgDauphin) -> Result<Track,Error> {
        let program = loader.get_program_model(&self.builder.program).await?;
        let t = self.builder.as_ref();
        Track::new(&program,&self.track_base,&self.track_mapping,t.scale_start,t.scale_end+1,t.scale_step,&t.tags)
    }

    pub(crate) fn mapping(&self) -> &TrackMapping { &self.track_mapping }

    pub(crate) fn mount_points(&self) -> Vec<(Vec<String>,bool)> {
        self.builder.triggers.iter().map(|x| (x.to_vec(),true)).collect()
    }
}

pub struct TrackModelDeserialize(pub BackendNamespace);

impl<'de> DeserializeSeed<'de> for TrackModelDeserialize {
    type Value = TrackModel;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where D: Deserializer<'de> {
        let builder = TrackModelBuilder::deserialize(deserializer)?;
        Ok(TrackModel::new(builder,&self.0))
    }
}
