use std::sync::Arc;
use peregrine_toolkit::{eachorevery::eoestruct::StructBuilt, error::Error };
use crate::{Track, shapeload::programname::ProgramName, PgDauphin };

pub struct TrackModelBuilder {
    name: String,
    program: ProgramName,
    tags: Vec<String>,
    triggers: Vec<Vec<String>>,
    extra: Vec<Vec<String>>,
    set: Vec<(Vec<String>,StructBuilt)>,
    settings: Vec<(String,Vec<String>)>,
    values: Vec<(String,StructBuilt)>,
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
            extra: vec![],
            set: vec![], 
            settings: vec![],
            values: vec![],
            scale_start, scale_end, scale_step
        }
    }

    pub fn add_setting(&mut self, key: &str, path: &[String]) {
        self.settings.push((key.to_string(),path.to_vec()));
    }

    pub fn add_value(&mut self, key: &str, value: StructBuilt) {
        self.values.push((key.to_string(),value));
    }

    pub fn add_tag(&mut self, tag: &str) { self.tags.push(tag.to_string()) }
    pub fn add_trigger(&mut self, trigger: &[String]) { self.triggers.push(trigger.to_vec()) }
    pub fn add_extra(&mut self, extra: &[String]) { self.extra.push(extra.to_vec()) }
    pub fn add_set(&mut self, set: &[String], value: StructBuilt) { 
        self.set.push((set.to_vec(),value));
    }
}

#[derive(Clone)]
pub struct TrackModel {
    builder: Arc<TrackModelBuilder>
}

impl TrackModel {
    pub fn new(builder: TrackModelBuilder) -> TrackModel {
        TrackModel {
            builder: Arc::new(builder)
        }
    }

    pub(crate) async fn to_track(&self, loader: &PgDauphin) -> Result<Track,Error> {
        let program = loader.get_program_model(&self.builder.program).await?;
        let t = self.builder.as_ref();
        let mut track = Track::new(&program,t.scale_start,t.scale_end+1,t.scale_step);
        for (key,value) in &t.set {
            let key = key.iter().map(|x| x.as_str()).collect::<Vec<_>>();
            track.set_switch(&key,value.clone());
        }
        Ok(track)
    }

    pub(crate) fn mount_points(&self) -> Vec<(Vec<String>,bool)> {
        let t = self.builder.as_ref();
        let mut out : Vec<_> = t.triggers.iter().map(|x| (x.to_vec(),true)).collect();
        out.append(&mut t.extra.iter().map(|x| (x.to_vec(),false)).collect());
        out
    }
}
