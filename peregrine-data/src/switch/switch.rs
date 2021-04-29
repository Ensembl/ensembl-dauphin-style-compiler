use std::sync::{ Arc, Mutex };
use std::collections::HashMap;
use super::trackconfig::{ TrackConfigList, TrackConfigNode };
use crate::lane::ProgramName;
use crate::switch::track::Track;

pub(crate) struct Switch {
    kids: HashMap<String,Switch>,
    set: bool,
    tracks: Vec<Track>,
    triggers: Vec<Track>
}

impl Switch {
    fn new() -> Switch {
        Switch {
            kids: HashMap::new(),
            set: false,
            tracks: vec![],
            triggers: vec![]
        }
    }

    fn get_or_make(&mut self, name: &str) -> &mut Switch {
        self.kids.entry(name.to_string()).or_insert_with(|| {
            Switch::new()
        })
    }

    fn get_target(&mut self, path: &[&str]) -> &mut Switch {
        if path.len() > 0 {
            self.get_or_make(&path[0]).get_target(&path[1..])
        } else {
            self
        }
    }

    pub(super) fn get_triggered(&self, out: &mut Vec<Track>) {
        if !self.set { return; }
        out.extend(self.triggers.iter().cloned());
        for kid in self.kids.values() {
            kid.get_triggered(out);
        }
    }

    fn new_active<'a>(&self, new_active: &'a mut Vec<Track>, active: &'a [Track]) -> &'a [Track] {
        let mut active = active;
        if self.tracks.len() > 0 {
            *new_active = active.to_vec();
            for track in &self.tracks {
                if !new_active.contains(track) {
                    new_active.push(track.clone());
                }
            }
            active = new_active;
        }
        active
    }

    fn add_nodes(&self, out: &mut HashMap<Track,TrackConfigNode>, path: &[String], active: &[Track]) {
        if active.len() > 0 {
            for track in active {
                if let Some(node) = out.get_mut(track) {
                    node.merge(path);
                }
            }
        }
    }

    pub(super) fn build_track_config_list(&self, out: &mut HashMap<Track,TrackConfigNode>, path: &mut Vec<String>, active: &[Track]) {
        let mut new_active = vec![];
        let new_active = self.new_active(&mut new_active,active);
        self.add_nodes(out,path,new_active);
        for (kid_name,kid) in &self.kids {
            if kid.set {
                path.push(kid_name.to_string());
                kid.build_track_config_list(out, path, new_active);
                path.pop();
            }
        }
    }
}

struct SwitchesData {
    root: Switch,
    track_config_list: Option<TrackConfigList>
}

impl SwitchesData {
    fn get_track_config_list(&mut self) -> &TrackConfigList {
        if self.track_config_list.is_none() {
            self.track_config_list = Some(TrackConfigList::new(&self.root));
        }
        self.track_config_list.as_ref().unwrap()
    }
}

#[derive(Clone)]
pub struct Switches(Arc<Mutex<SwitchesData>>);

impl Switches {
    pub fn new() -> Switches {
        let out = Switches(Arc::new(Mutex::new(SwitchesData {
            root: Switch::new(),
            track_config_list: None
        })));
        out.set_switch(&[]);
        out
    }

    pub fn set_switch(&self, path: &[&str]) {
        let mut data = self.0.lock().unwrap();        
        data.root.get_target(path).set = true;
        data.track_config_list = None;
    }

    pub fn clear_switch(&self, path: &[&str]) {
        let mut data = self.0.lock().unwrap();
        data.root.get_target(path).set = false;        
        data.track_config_list = None;
    }

    pub fn add_track(&self, path: &[&str], lane_name: &str, track: &Track, trigger: bool) {
        let mut data = self.0.lock().unwrap();
        let target = data.root.get_target(path);
        target.tracks.push(track.clone());
        if trigger {
            target.triggers.push(track.clone());
        }
        data.track_config_list = None;
    }

    pub fn get_track_config_list(&self) -> TrackConfigList {
        let mut data = self.0.lock().unwrap();
        data.get_track_config_list().clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{ Channel, ChannelLocation };
    use url::Url;

    fn urlc(idx: u32) -> Url {
        Url::parse(&(format!("http://a.com/{}",idx))).expect("b")
    }    

    fn list_tracks(track_config_list: &TrackConfigList) -> Vec<String> {
        let tracks = track_config_list.list_tracks();
        tracks.iter().map(|x| x.program_name().1.clone()).collect()
    }

    #[test]
    fn switch_smoke() {
        let program_a = ProgramName(Channel::new(&ChannelLocation::HttpChannel(urlc(1))),"A".to_string());
        let program_b = ProgramName(Channel::new(&ChannelLocation::HttpChannel(urlc(1))),"B".to_string());
        let track_a = Track::new(&program_a);
        let track_b = Track::new(&program_b);
        let switches = Switches::new();
        switches.add_track(&["track","A"],"A",&track_a,true);
        switches.add_track(&["general"],"A",&track_a,false);
        switches.add_track(&["track","B"],"B",&track_b,true);
        switches.add_track(&["general"],"B",&track_b,false);
        switches.set_switch(&["general"]);
        switches.set_switch(&["track"]);
        switches.set_switch(&["track","B"]);
        switches.set_switch(&["track","B","normal"]);
        let track_config_list = switches.get_track_config_list();
        assert_eq!(vec!["B"],list_tracks(&track_config_list));
        let track_b_data = track_config_list.get_track(&track_b).unwrap();
        assert_eq!(true,track_b_data.contains(&["track"]));
        assert_eq!(Some(vec!["B".to_string()]),track_b_data.get(&["track"]));
        assert_eq!(true,track_b_data.contains(&["general"]));
        assert_eq!(Some(vec![]),track_b_data.get(&["general"]));
        assert_eq!(true,track_b_data.contains(&["track","B"]));
        assert_eq!(Some(vec!["normal".to_string()]),track_b_data.get(&["track","B"]));
        assert_eq!(false,track_b_data.contains(&["missing"]));
        assert_eq!(None,track_b_data.get(&["missing"]));
        /* check modification */
        switches.set_switch(&["track","A"]);
        let track_config_list = switches.get_track_config_list();
        assert_eq!(true,list_tracks(&track_config_list).contains(&"A".to_string()));
        assert_eq!(true,list_tracks(&track_config_list).contains(&"B".to_string()));
    }
}
