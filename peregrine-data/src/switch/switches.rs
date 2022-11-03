use std::sync::{Arc, Mutex};

use peregrine_toolkit::{eachorevery::eoestruct::{StructTemplate, StructBuilt, StructConst}, lock, error::Error};

use crate::{Track, request::tracks::{trackmodel::TrackModel, expansionmodel::ExpansionModel}, AllBackends, PgDauphin};

use super::{trackconfiglist::TrackConfigList, switch::Switch, trackconfig::TrackConfigNode, expansion::Expansion};

pub(super) struct SwitchesData {
    root: Switch,
    all_backends: Option<AllBackends>,
    track_config_list: Option<TrackConfigList>
}

impl SwitchesData {
    fn new() -> SwitchesData {
        let mut out = SwitchesData {
            root: Switch::new(),
            all_backends: None,
            track_config_list: None
        };
        let tmpl_true = StructTemplate::new_boolean(true).build().ok().unwrap();
        out.root.set(tmpl_true);
        out
    }

    fn set_all_backends(&mut self, all_backends: &AllBackends) {
        self.all_backends = Some(all_backends.clone());
    }

    fn get_track_config_list(&mut self) -> &TrackConfigList {
        if self.track_config_list.is_none() {
            self.track_config_list = Some(TrackConfigList::new(&self));
        }
        self.track_config_list.as_ref().unwrap()
    }

    pub(super) fn get_triggered(&self) -> Vec<Track> {
        let mut triggered = vec![];
        self.root.get_triggered(&mut triggered);
        triggered
    }

    pub(super) fn build_track_config(&self, track: &Track) -> TrackConfigNode {
        let mut out = TrackConfigNode::empty();
        let overlay = track.overlay();
        self.root.build_track_config(track, &mut out, &mut vec![], false,&overlay,true);
        overlay.apply(&mut out);
        out
    }
}

#[derive(Clone)]
pub struct Switches {
    data: Arc<Mutex<SwitchesData>>,
}

impl Switches {
    pub fn new() -> Switches {
        Switches{
            data: Arc::new(Mutex::new(SwitchesData::new()))
        }
    }

    pub fn set_all_backends(&mut self, all_backends: &AllBackends) {
        lock!(self.data).set_all_backends(all_backends);
    }

    async fn run_expansions(&self, path: &[&str]) -> Result<(),Error> {
        let data = lock!(self.data);
        let all_backends = data.all_backends.clone().expect("missing all_backends");
        drop(data);
        for len in 0..path.len() {
            let mut data = lock!(self.data);
            let expansions = data.root.get_target(&path[0..len]).find_expansions().to_vec();
            drop(data);
            for expansion in &expansions {
                expansion.run(&all_backends,path[len]).await?;
            }    
        }
        Ok(())
    }

    pub async fn switch(&self, path: &[&str], value: StructBuilt) -> Result<(),Error> {
        self.run_expansions(path).await?;
        let mut data = lock!(self.data);
        if value.truthy() {
            /* unset radio siblings */
            if path.len() > 0 {
                let parent = data.root.get_target(&path[0..(path.len()-1)]);
                parent.clear_if_radio();
            }
        }
        if value == StructBuilt::Const(StructConst::Null) {
            data.root.remove(path);
        } else {
            let target = data.root.get_target(path);
            target.set(value);
        }
        data.track_config_list = None;
        Ok(())
    }

    pub fn radio_switch(&self, path: &[&str], yn: bool) {
        let mut data = lock!(self.data);
        data.root.get_target(path).set_radio(yn);        
        data.track_config_list = None;
    }

    fn add_track(&self, path: &[&str], track: &Track, trigger: bool) {
        let mut data = lock!(self.data);
        let target = data.root.get_target(path);
        target.add_track(track,trigger);
        data.track_config_list = None;
    }

    pub async fn add_track_model(&self, model: &TrackModel, pgd: &PgDauphin) -> Result<(),Error> {
        let track = model.to_track(pgd).await?;
        for (mount,trigger) in model.mount_points() {
            let path = mount.iter().map(|x| x.as_str()).collect::<Vec<_>>();
            self.add_track(&path,&track,trigger);
        }
        Ok(())
    }

    fn add_expansion(&self, path: &[&str], expansion: &Expansion) {
        let mut data = lock!(self.data);
        let target = data.root.get_target(path);
        target.add_expansion(expansion);
        data.track_config_list = None;        
    }

    pub fn add_expansion_model(&self, model: &ExpansionModel) {
        let expansion = model.to_expansion();
        for trigger in model.triggers() {
            let path = trigger.iter().map(|x| x.as_str()).collect::<Vec<_>>();
            self.add_expansion(&path,&expansion);
        }
    }

    pub fn get_track_config_list(&self) -> TrackConfigList {
        lock!(self.data).get_track_config_list().clone()
    }
}
