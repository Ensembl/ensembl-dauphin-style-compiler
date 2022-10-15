use std::sync::Arc;

use crate::{Track, ProgramName};

#[derive(Debug)]
pub struct TrackModelBuilder {
    name: String,
    program: ProgramName,
    tags: Vec<String>,
    triggers: Vec<Vec<String>>,
    extra: Vec<Vec<String>>,
    set: Vec<Vec<String>>,
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
            scale_start, scale_end, scale_step
        }
    }

    pub fn add_tag(&mut self, tag: &str) { self.tags.push(tag.to_string()) }
    pub fn add_trigger(&mut self, trigger: &[String]) { self.triggers.push(trigger.to_vec()) }
    pub fn add_extra(&mut self, extra: &[String]) { self.extra.push(extra.to_vec()) }
    pub fn add_set(&mut self, set: &[String]) { self.set.push(set.to_vec()) }
}

#[derive(Debug)]
pub struct TrackModel(Arc<TrackModelBuilder>);

impl TrackModel {
    pub fn new(builder: TrackModelBuilder) -> TrackModel {
        TrackModel(Arc::new(builder))
    }

    pub(crate) fn to_track(&self) -> Track {
        let t = self.0.as_ref();
        Track::new(&t.program,t.scale_start,t.scale_end+1,t.scale_step)
    }

    pub(crate) fn mount_points(&self) -> Vec<(Vec<String>,bool)> {
        let t = self.0.as_ref();
        let mut out : Vec<_> = t.triggers.iter().map(|x| (x.to_vec(),true)).collect();
        out.append(&mut t.extra.iter().map(|x| (x.to_vec(),false)).collect());
        out
    }
}
