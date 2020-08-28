use peregrine_core::{ lock, PanelSliceRange };
use anyhow::{ anyhow as err };
use std::collections::HashMap;
use std::sync::{ Arc, Mutex };

struct PanelBuilderData {
    next_id: usize,
    panels: HashMap<usize,Arc<Mutex<PanelSliceRange>>>
}

impl PanelBuilderData {
    fn new() -> PanelBuilderData {
        PanelBuilderData {
            next_id: 0,
            panels: HashMap::new()
        }
    }

    fn allocate(&mut self) -> usize {
        let psr = Arc::new(Mutex::new(PanelSliceRange::new()));
        let id = self.next_id;
        self.panels.insert(id,psr.clone());
        self.next_id += 1;
        id
    }

    fn get(&self, id: usize) -> anyhow::Result<Arc<Mutex<PanelSliceRange>>> {
        Ok(self.panels.get(&id).ok_or(err!("bad panel id"))?.clone())
    }
}

#[derive(Clone)]
pub struct PanelBuilder(Arc<Mutex<PanelBuilderData>>);

impl PanelBuilder {
    pub fn new() -> PanelBuilder {
        PanelBuilder(Arc::new(Mutex::new(PanelBuilderData::new())))
    }

    pub fn allocate(&self) -> usize {
        lock!(self.0).allocate()
    }

    pub fn get(&self, id: usize) -> anyhow::Result<Arc<Mutex<PanelSliceRange>>> {
        lock!(self.0).get(id)
    }
}