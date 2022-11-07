use std::{sync::Arc, collections::{BTreeMap, HashMap}};
use peregrine_toolkit::{eachorevery::eoestruct::StructBuilt, error::Error, log };
use crate::{Track, shapeload::programname::ProgramName, PgDauphin, switch::switches::SwitchesData };

pub(crate) struct TrackMappingBuilder {
    settings: HashMap<String,Vec<String>>,
    values: Vec<(String,StructBuilt)>,
}

impl TrackMappingBuilder {
    fn new() -> TrackMappingBuilder {
        TrackMappingBuilder {
            settings: HashMap::new(),
            values: vec![]
        }
    }

    pub(crate) fn add_setting(&mut self, key: &str, path: &[String]) {
        self.settings.insert(key.to_string(),path.to_vec());
    }

    pub(crate) fn add_value(&mut self, key: &str, value: StructBuilt) {
        self.values.push((key.to_string(),value));
    }
}

#[derive(Clone)]
pub struct TrackMapping(Arc<TrackMappingBuilder>);

impl TrackMapping {
    fn new(builder: TrackMappingBuilder) -> TrackMapping {
        TrackMapping(Arc::new(builder))
    }

    pub(crate) fn apply(&self, switches_data: &SwitchesData) -> BTreeMap<String,StructBuilt> {
        let mut out = BTreeMap::new();
        for (key,value) in &self.0.values {
            out.insert(key.to_string(),value.clone());
        }
        for (key,path) in &self.0.settings {
            let path_str = path.iter().map(|x| x.as_str()).collect::<Vec<_>>();
            out.insert(key.to_string(),switches_data.get_value(&path_str));
        }
        out
    }

    pub fn get_switch(&self, setting: &str) -> Option<&[String]> {
        log!("get {:?} from {:?}",setting,self.0.settings);
        self.0.settings.get(setting).map(|x| x.as_slice())
    }
}

pub struct TrackModelBuilder {
    name: String,
    program: ProgramName,
    tags: Vec<String>,
    triggers: Vec<Vec<String>>,
    mapping: Option<TrackMappingBuilder>,
    scale_start: u64,
    scale_end: u64,
    scale_step: u64
}

impl TrackModelBuilder {
    pub fn new(name: &str, program: &ProgramName, scale_start: u64, scale_end: u64, scale_step: u64) -> TrackModelBuilder {
        TrackModelBuilder {
            name: name.to_string(),
            program: program.clone(),
            tags: vec![], 
            triggers: vec![],
            mapping: Some(TrackMappingBuilder::new()),
            scale_start, scale_end, scale_step
        }
    }

    pub(crate) fn mapping_mut(&mut self) -> &mut TrackMappingBuilder { self.mapping.as_mut().unwrap() }
    pub fn add_tag(&mut self, tag: &str) { self.tags.push(tag.to_string()) }
    pub fn add_trigger(&mut self, trigger: &[String]) { self.triggers.push(trigger.to_vec()) }
}

#[derive(Clone)]
pub struct TrackModel {
    builder: Arc<TrackModelBuilder>,
    track_mapping: TrackMapping
}

impl TrackModel {
    pub fn new(mut builder: TrackModelBuilder) -> TrackModel {
        TrackModel {
            track_mapping: TrackMapping::new(builder.mapping.take().unwrap()),
            builder: Arc::new(builder)
        }
    }

    pub(crate) async fn to_track(&self, loader: &PgDauphin) -> Result<Track,Error> {
        let program = loader.get_program_model(&self.builder.program).await?;
        let t = self.builder.as_ref();
        Ok(Track::new(&program,&self.track_mapping,t.scale_start,t.scale_end+1,t.scale_step))
    }

    pub(crate) fn mount_points(&self) -> Vec<(Vec<String>,bool)> {
        self.builder.triggers.iter().map(|x| (x.to_vec(),true)).collect()
    }
}
