use crate::lock;
use std::sync::{ Arc, Mutex };
use varea::VareaStore;
use crate::request::Channel;
use super::panel::PanelProgramRegion;

pub struct PanelProgramStoreData {
    store: VareaStore<(Channel,String,PanelProgramRegion)>
}

impl PanelProgramStoreData {
    fn new() -> PanelProgramStoreData {
        PanelProgramStoreData {
            store: VareaStore::new()
        }
    }

    fn add(&mut self, panel_slice_range: &PanelProgramRegion, channel: &Channel, name: &str) {
        let varea_item = panel_slice_range.to_varea_item();
        self.store.add(varea_item,(channel.clone(),name.to_string(),panel_slice_range.clone()));
    }

    fn get(&self, panel_slice_range: &PanelProgramRegion) -> Option<(Channel,String,PanelProgramRegion)> {
        let varea_item = panel_slice_range.to_varea_item();
        let varea_search_term = self.store.search_item(&varea_item);
        let mut varea_matches = self.store.lookup(varea_search_term);
        varea_matches.next().map(|(c,n,ppr)| (c.clone(),n.to_string(),ppr.clone()))
    }
}

#[derive(Clone)]
pub struct PanelProgramStore(Arc<Mutex<PanelProgramStoreData>>);

impl PanelProgramStore {
    pub fn new() -> PanelProgramStore {
        PanelProgramStore(Arc::new(Mutex::new(PanelProgramStoreData::new())))
    }

    pub fn add(&self, panel_slice_range: &PanelProgramRegion, channel: &Channel, name: &str) {
        lock!(self.0).add(panel_slice_range,channel,name);
    }

    pub fn get(&self, panel_slice_range: &PanelProgramRegion) -> Option<(Channel,String,PanelProgramRegion)> {
        lock!(self.0).get(panel_slice_range)
    }
}
