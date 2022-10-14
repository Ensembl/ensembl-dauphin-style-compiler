use std::sync::Arc;

#[derive(Debug)]
pub struct TrackModelBuilder {
    name: String,
    program: String,
    tags: Vec<String>,
    triggers: Vec<Vec<String>>,
    extra: Vec<Vec<String>>,
    set: Vec<Vec<String>>,
    scale_start: usize,
    scale_end: usize,
    scale_step: usize
}

impl TrackModelBuilder {
    pub fn new(name: &str, program: &str, scale_start: usize, scale_end: usize, scale_step: usize) -> TrackModelBuilder {
        TrackModelBuilder {
            name: name.to_string(),
            program: program.to_string(),
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
}
