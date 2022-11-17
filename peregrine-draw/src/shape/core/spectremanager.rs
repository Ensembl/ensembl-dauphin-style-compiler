use std::sync::{ Arc, Mutex };
use keyed::{KeyedOptionalValues, keyed_handle };
use peregrine_data::{Assets, reactive::Reactive};
use peregrine_toolkit_async::{sync::needed::{Needed, NeededLock}};
use peregrine_toolkit::lock;
use crate::{Message, run::PgPeregrineConfig, stage::stage::ReadStage, webgl::{DrawingSession, global::WebGlGlobal}, PgCommanderWeb, shape::spectres::{spectre::Spectre, ants::{AreaVariables2, MarchingAnts}, stain::Stain}};
use super::{spectraldrawing::SpectralDrawing};

#[derive(Clone,PartialEq,Eq,Hash)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub enum SpectreConfigKey {
    MarchingAntsWidth,
    MarchingAntsColour,
    MarchingAntsLength,
    MarchingAntsProp,
    StainColour,
}

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
pub(crate) struct SpectreManager {
    state: Arc<Mutex<SpectreState>>,
    drawing: SpectralDrawing,
    config: Arc<PgPeregrineConfig>
}

impl SpectreManager {
    pub(crate) fn new(commander: &PgCommanderWeb, config: &Arc<PgPeregrineConfig>, redraw_needed: &Needed) -> SpectreManager {
        let reactive = Reactive::new();
        SpectreManager {
            state: Arc::new(Mutex::new(SpectreState::new(redraw_needed))),
            drawing: SpectralDrawing::new(commander,&reactive),
            config: config.clone()
        }
    }

    pub(crate) fn marching_ants(&self, area2: &AreaVariables2<'static>) -> Result<Spectre,Message> {
        Ok(Spectre::MarchingAnts(MarchingAnts::new(&self.config,area2)?))
    }

    pub(crate) fn stain(&self, area2: &AreaVariables2<'static>, flip: bool) -> Result<Spectre,Message> {
        Ok(Spectre::Stain(Stain::new(&self.config,area2,flip)?))
    }

    pub(crate) fn add(&self, spectre: Spectre) -> SpectreHandle {
        let id = self.state.lock().unwrap().add(spectre);
        SpectreHandle(self.state.clone(),id)
    }

    pub(crate) fn get_spectres(&self) -> Vec<Spectre> {
        self.state.lock().unwrap().get_spectres()        
    }

    pub(crate) fn draw(&mut self, gl: &Arc<Mutex<WebGlGlobal>>, assets: &Assets, stage: &ReadStage, session: &mut DrawingSession) -> Result<(),Message> {
        if self.state.lock().unwrap().new_shapes() {
            self.drawing.set(gl,assets,&self.get_spectres());
        }
        self.drawing.draw(&mut *lock!(gl),stage,session)
    }

    pub(crate) fn update(&self, gl: &WebGlGlobal) -> Result<(),Message> { self.drawing.update(gl) }
    pub(crate) fn reactive(&self) -> &Reactive<'static> { self.drawing.reactive() }
}
