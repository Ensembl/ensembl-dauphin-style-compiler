use varea::VareaStore;
use crate::request::Channel;
use super::panel::PanelSliceRange;

pub struct PanelProgramStore {
    store: VareaStore<(Channel,String)>
}

impl PanelProgramStore {
    pub fn new() -> PanelProgramStore {
        PanelProgramStore {
            store: VareaStore::new()
        }
    }

    pub fn add(&mut self, panel_slice_range: &PanelSliceRange, channel: &Channel, name: &str) {
        let varea_item = panel_slice_range.to_varea_item();
        self.store.add(varea_item,(channel.clone(),name.to_string()));
    }

    pub fn get(&self, panel_slice_range: &PanelSliceRange) -> Option<(Channel,String)> {
        let varea_item = panel_slice_range.to_varea_item();
        let varea_search_term = self.store.search_item(&varea_item);
        let mut varea_matches = self.store.lookup(varea_search_term);
        varea_matches.next().map(|(c,n)| (c.clone(),n.to_string()))
    }
}