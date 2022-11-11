use std::{sync::{Arc, Mutex}, collections::HashSet};
use peregrine_toolkit::{eachorevery::eoestruct::{ StructConst, StructValue}, lock, error::Error, log};
use crate::{request::tracks::{trackmodel::TrackModel, expansionmodel::ExpansionModel}, AllBackends, PgDauphin, SettingMode};
use super::{trackconfiglist::{TrackConfigList, TrackConfigListBuilder}, switch::Switch, expansion::Expansion};

pub(crate) struct SwitchesData {
    root: Switch,
    all_backends: Option<AllBackends>,
    track_config_list_builder: Option<TrackConfigListBuilder>
}

impl SwitchesData {
    fn new() -> SwitchesData {
        let mut out = SwitchesData {
            root: Switch::new(),
            all_backends: None,
            track_config_list_builder: None
        };
        out.root.set(StructValue::new_boolean(true));
        out
    }

    pub(crate) fn get_value(&self, path: &[&str]) -> StructValue {
        self.root.get_value(path).clone()
    }
    
    fn set_all_backends(&mut self, all_backends: &AllBackends) {
        self.all_backends = Some(all_backends.clone());
    }

    fn get_track_config_list_builder(&mut self, loader: &PgDauphin) -> &TrackConfigListBuilder {
        if self.track_config_list_builder.is_none() {
            self.track_config_list_builder = Some(TrackConfigListBuilder::new(self,loader))
        }
        self.track_config_list_builder.as_ref().unwrap()
    }

    pub(super) fn get_triggered(&self) -> HashSet<TrackModel> {
        let mut triggered = HashSet::new();
        self.root.get_triggered(&mut triggered);
        triggered
    }

    fn switch_inner(&mut self, path: &[&str], value: StructValue) {
        if value.truthy() {
            /* unset radio siblings */
            if path.len() > 0 {
                let parent = self.root.get_target(&path[0..(path.len()-1)]);
                parent.clear_if_radio();
            }
        }
        if value == StructValue::Const(StructConst::Null) {
            self.root.remove(path);
        } else {
            let target = self.root.get_target(path);
            target.set(value);
        }
        self.track_config_list_builder = None;
    }
}

#[derive(Clone)]
pub struct Switches {
    data: Arc<Mutex<SwitchesData>>,
    loader: PgDauphin
}

impl Switches {
    pub fn new(loader: &PgDauphin) -> Switches {
        Switches {
            data: Arc::new(Mutex::new(SwitchesData::new())),
            loader: loader.clone()
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

    pub async fn switch(&self, path: &[&str], value: StructValue) -> Result<(),Error> {
        self.run_expansions(path).await?;
        let mut data = lock!(self.data);
        data.switch_inner(path,value);
        Ok(())
    }

    pub async fn update_switch(&self, path: &[&str], value: SettingMode) -> Result<(),Error> {
        self.run_expansions(path).await?;
        let mut data = lock!(self.data);
        let new = value.update(data.get_value(path));
        data.switch_inner(path,new);
        Ok(())
    }

    pub fn radio_switch(&self, path: &[&str], yn: bool) {
        let mut data = lock!(self.data);
        data.root.get_target(path).set_radio(yn);        
        data.track_config_list_builder = None;
    }

    fn add_track(&self, path: &[&str], track: &TrackModel, trigger: bool) {
        let mut data = lock!(self.data);
        let target = data.root.get_target(path);
        target.add_track(track,trigger);
        data.track_config_list_builder = None;
    }

    pub fn add_track_model(&self, model: &TrackModel, pgd: &PgDauphin) -> Result<(),Error> {
        for (mount,trigger) in model.mount_points() {
            let path = mount.iter().map(|x| x.as_str()).collect::<Vec<_>>();
            self.add_track(&path,&model,trigger);
        }
        Ok(())
    }

    fn add_expansion(&self, path: &[&str], expansion: &Expansion) {
        let mut data = lock!(self.data);
        let target = data.root.get_target(path);
        target.add_expansion(expansion);
        data.track_config_list_builder = None;        
    }

    pub fn add_expansion_model(&self, model: &ExpansionModel) {
        let expansion = model.to_expansion();
        for trigger in model.triggers() {
            let path = trigger.iter().map(|x| x.as_str()).collect::<Vec<_>>();
            self.add_expansion(&path,&expansion);
        }
    }

    /* Building is async, by which time we mayhave another builder so may need to retry. */
    pub async fn get_track_config_list(&self) -> Result<TrackConfigList,Error> {
        let mut out : Option<TrackConfigList> = None;
        loop {
            let mut data = lock!(self.data);
            let builder = data.get_track_config_list_builder(&self.loader).clone();
            if let Some(out) = &out {
                if out.compatible_with(&builder) {
                    return Ok(out.clone());
                }
            }
            drop(data);
            out = Some(builder.track_config_list().await?);
        }
    }
}
