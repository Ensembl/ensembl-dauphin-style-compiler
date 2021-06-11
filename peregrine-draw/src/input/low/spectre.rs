use std::sync::{ Arc, Mutex };
use keyed::{KeyedOptionalValues, keyed_handle};

use crate::util::needed::{Needed, NeededLock};

pub struct SpectreHandle(Arc<Mutex<SpectreState>>,SpectreId);

impl Drop for SpectreHandle {
    fn drop(&mut self) {
        self.0.lock().unwrap().free(&self.1);
    }
}

keyed_handle!(SpectreId);

#[derive(Clone,Debug)]
pub(crate) enum Spectre {
    MarchingAnts
}

struct SpectreState {
    spectres: KeyedOptionalValues<SpectreId,Spectre>,
    redraw_needed: Needed,
    redraw_lock: Option<NeededLock>
}

impl SpectreState {
    pub(crate) fn new(redraw_needed: &Needed) -> SpectreState {
        SpectreState {
            spectres: KeyedOptionalValues::new(),
            redraw_needed: redraw_needed.clone(),
            redraw_lock: None
        }
    }

    pub(crate) fn add(&mut self, spectre: Spectre) -> SpectreId {
        if self.redraw_lock.is_none() {
            self.redraw_lock = Some(self.redraw_needed.lock());
        }
        self.spectres.add(spectre)
    }

    pub(crate) fn get_spectres(&self) -> Vec<Spectre> {
        self.spectres.keys()
            .map(|id| self.spectres.get(&id).map(|x| x.clone()))
            .filter_map(|x| x.ok())
            .collect()
    }

    fn free(&mut self, id: &SpectreId) {
        self.spectres.remove(id);
        if self.spectres.size() == 0 {
            self.redraw_lock = None;
        }
    }
}

#[derive(Clone)]
pub(crate) struct SpectreManager(Arc<Mutex<SpectreState>>);

impl SpectreManager {
    pub(crate) fn new(redraw_needed: &Needed) -> SpectreManager {
        SpectreManager(Arc::new(Mutex::new(SpectreState::new(redraw_needed))))
    }

    pub(crate) fn add(&self, spectre: Spectre) -> SpectreHandle {
        let id = self.0.lock().unwrap().add(spectre);
        SpectreHandle(self.0.clone(),id)
    }

    pub(crate) fn get_spectres(&self) -> Vec<Spectre> {
        self.0.lock().unwrap().get_spectres()        
    }
}
