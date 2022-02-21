use crate::core::{ Scale };
use std::cmp::max;
use crate::{ Switches, Track };

pub struct ProgramRegionBuilder {
    program_region: ProgramRegion,
    mounts: Vec<(Vec<String>,bool)>,
    switches: Vec<(Vec<String>,bool)>
}

impl ProgramRegionBuilder {
    pub fn new() -> ProgramRegionBuilder {
        ProgramRegionBuilder {
            program_region: ProgramRegion::new(),
            mounts: vec![],
            switches: vec![]
        }
    }

    pub fn program_region_mut(&mut self) -> &mut ProgramRegion { &mut self.program_region }

    pub fn add_mount(&mut self, path: &[&str], trigger: bool) {
        let path = path.iter().map(|x| x.to_string()).collect();
        self.mounts.push((path,trigger));
    }

    pub fn add_switch(&mut self, path: &[&str], yn: bool) {
        let path = path.iter().map(|x| x.to_string()).collect();
        self.switches.push((path,yn));
    }

    pub fn build(&mut self, track: &Track, switches: &Switches) -> ProgramRegion {
        for (path,trigger) in &self.mounts {
            let path : Vec<_> = path.iter().map(|x| x.as_str()).collect();
            switches.add_track(&path,track,*trigger);
        }
        let scale = track.scale();
        self.program_region.set_scale(Scale::new(scale.0),Scale::new(scale.1));
        self.program_region.set_max_scale_jump(track.max_scale_jump() as u32);
        self.program_region.set_stick_tags(track.tags());
        self.program_region.set_switches(&self.switches);
        self.program_region.clone()
    }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct ProgramRegion {
    stick_tags: Option<Vec<String>>,
    scale: Option<(Scale,Scale)>,
    track: Option<Vec<String>>,
    switches: Vec<(Vec<String>,bool)>,
    max_scale_jump: Option<u64>
}

impl ProgramRegion {
    pub fn new() -> ProgramRegion {
        ProgramRegion {
            stick_tags: None,
            scale: None,
            track: None,
            switches: vec![],
            max_scale_jump: None
        }
    }

    pub fn stick_tags(&self) -> Option<&[String]> { self.stick_tags.as_ref().and_then(|x| Some(x.as_ref())) }
    pub fn tracks(&self) -> Option<&[String]> { self.track.as_ref().and_then(|x| Some(x.as_ref())) }
    pub fn scale(&self) -> Option<(&Scale,&Scale)> { self.scale.as_ref().and_then(|x| Some((&x.0,&x.1))) }
    pub fn max_scale_jump(&self) -> Option<u32> { self.max_scale_jump.map(|x| x as u32) }
    pub fn switches(&self) ->  &[(Vec<String>,bool)] { &self.switches }

    pub fn set_stick_tags(&mut self, stick_tags: &[String]) { self.stick_tags = Some(stick_tags.to_vec()); }
    pub fn set_scale(&mut self, a: Scale, b: Scale) { self.scale = Some((a,b)); }
    pub fn set_tracks(&mut self, t: &[String]) {  self.track = Some(t.to_vec()); }
    pub fn set_max_scale_jump(&mut self, jump: u32) { self.max_scale_jump = Some(jump as u64); }
    pub fn set_switches(&mut self, switches: &[(Vec<String>,bool)]) { self.switches = switches.to_vec(); }

    pub fn scale_up(&self, input: &Scale) -> Scale {
        if let Some(scale_range) = &self.scale {
            if let Some(max_jump) = self.max_scale_jump {
                let input = input.get_index();
                let last_idx = scale_range.1.delta_scale(-1).map(|s| s.get_index()).unwrap_or(0);
                let deficit = last_idx - input;
                let deficit = (deficit/max_jump) * max_jump;
                let output = max(scale_range.0.get_index(),last_idx-deficit);
                Scale::new(output)
            } else {
                scale_range.1.delta_scale(-1).unwrap_or_else(|| Scale::new(0))
            }
        } else {
            Scale::new(100)
        }
    }    
}
