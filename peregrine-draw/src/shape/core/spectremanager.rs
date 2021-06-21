use std::sync::{ Arc, Mutex };
use keyed::{KeyedOptionalValues, keyed_handle, KeyedHandle };
use peregrine_data::AllotmentPetitioner;

use crate::{Message, stage::stage::ReadStage, util::needed::{Needed, NeededLock}, webgl::{DrawingSession, global::WebGlGlobal}};

use super::{spectraldrawing::SpectralDrawing, spectre::Spectre};

pub struct SpectreHandle(Arc<Mutex<SpectreState>>,SpectreId);

impl SpectreHandle {
    pub(crate) fn update(&self, spectre: Spectre) {
        self.0.lock().unwrap().update(&self.1,spectre);
    }
}

impl Drop for SpectreHandle {
    fn drop(&mut self) {
        self.0.lock().unwrap().free(&self.1);
    }
}

keyed_handle!(SpectreId);

struct SpectreState {
    spectres: KeyedOptionalValues<SpectreId,Spectre>,
    new_shapes: Needed,
    redraw_needed: Needed,
    redraw_lock: Option<NeededLock>
}

impl SpectreState {
    pub(crate) fn new(redraw_needed: &Needed) -> SpectreState {
        SpectreState {
            spectres: KeyedOptionalValues::new(),
            new_shapes: Needed::new(),
            redraw_needed: redraw_needed.clone(),
            redraw_lock: None
        }
    }

    pub(crate) fn add(&mut self, spectre: Spectre) -> SpectreId {
        self.new_shapes.set();
        if self.redraw_lock.is_none() {
            self.redraw_lock = Some(self.redraw_needed.lock());
        }
        self.spectres.add(spectre)
    }

    pub(crate) fn update(&mut self, handle: &SpectreId, spectre: Spectre) {
        self.new_shapes.set();
        self.spectres.replace(handle,spectre).ok();
    }

    pub(crate) fn get_spectres(&self) -> Vec<Spectre> {
        self.spectres.keys()
            .map(|id| self.spectres.get(&id).map(|x| x.clone()))
            .filter_map(|x| x.ok())
            .collect()
    }

    fn free(&mut self, id: &SpectreId) {
        self.new_shapes.set();
        self.spectres.remove(id);
        if self.spectres.size() == 0 {
            self.redraw_lock = None;
        }
    }

    fn new_shapes(&mut self) -> bool {
        self.new_shapes.is_needed()
    }
}

#[derive(Clone)]
pub(crate) struct SpectreManager(Arc<Mutex<SpectreState>>,SpectralDrawing);

impl SpectreManager {
    pub(crate) fn new(redraw_needed: &Needed) -> SpectreManager {
        SpectreManager(Arc::new(Mutex::new(SpectreState::new(redraw_needed))),SpectralDrawing::new())
    }

    pub(crate) fn add(&self, spectre: Spectre) -> SpectreHandle {
        let id = self.0.lock().unwrap().add(spectre);
        SpectreHandle(self.0.clone(),id)
    }

    pub(crate) fn get_spectres(&self) -> Vec<Spectre> {
        self.0.lock().unwrap().get_spectres()        
    }

    pub(crate) fn draw(&mut self, allotment_petitioner: &mut AllotmentPetitioner, gl: &mut WebGlGlobal, stage: &ReadStage, session: &DrawingSession) -> Result<(),Message> {
        if self.0.lock().unwrap().new_shapes() {
            self.1.update(gl,allotment_petitioner,&self.get_spectres())?;
        }
        self.1.draw(gl,stage,session)
    }
}
