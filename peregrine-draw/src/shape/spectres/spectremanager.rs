use std::sync::{ Arc, Mutex };
use keyed::{KeyedOptionalValues, keyed_handle };
use peregrine_data::{Assets, reactive::Reactive};
use peregrine_toolkit_async::{sync::needed::{Needed, NeededLock}};
use peregrine_toolkit::lock;
use crate::{Message, run::PgPeregrineConfig, stage::stage::ReadStage, webgl::{DrawingSession, global::WebGlGlobal}, PgCommanderWeb, shape::spectres::{spectre::{AreaVariables}, ants::{MarchingAnts}, stain::Stain}};
use super::{spectraldrawing::SpectralDrawing, spectre::{Spectre}};

#[derive(Clone,PartialEq,Eq,Hash)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub enum SpectreConfigKey {
    MarchingAntsWidth,
    MarchingAntsColour,
    MarchingAntsLength,
    MarchingAntsProp,
    StainColour,
}

// XXX to toolkit
pub(crate) struct SpectreHandle(Arc<Mutex<SpectreState>>,SpectreId);

impl Drop for SpectreHandle {
    fn drop(&mut self) {
        lock!(self.0).free(&self.1);
    }
}

keyed_handle!(SpectreId);

struct SpectreState {
    spectres: KeyedOptionalValues<SpectreId,Arc<dyn Spectre>>,
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

    fn add(&mut self, spectre: Arc<dyn Spectre>) -> SpectreId {
        self.new_shapes.set();
        if self.redraw_lock.is_none() {
            self.redraw_lock = Some(self.redraw_needed.lock());
        }
        self.spectres.add(spectre)
    }

    fn get_spectres(&self) -> Vec<Arc<dyn Spectre>> {
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

    pub(crate) fn marching_ants(&mut self, area2: &AreaVariables<'static>) -> Result<SpectreHandle,Message> {
        let (ants,handle) = MarchingAnts::new(&self.config,area2,&self)?;
        Ok(handle)
    }

    pub(crate) fn stain(&mut self, area2: &AreaVariables<'static>, flip: bool) -> Result<SpectreHandle,Message> {
        let (stain,handle) = Stain::new(&self.config,area2,&self,flip)?;
        Ok(handle)
    }

    pub(crate) fn add<X>(&self, spectre: &Arc<X>) -> SpectreHandle where X: Spectre + 'static {
        let id = lock!(self.state).add(spectre.clone());
        SpectreHandle(self.state.clone(),id)
    }

    pub(crate) fn active(&self) -> bool {
        self.get_spectres().len() > 0
    }

    fn get_spectres(&self) -> Vec<Arc<dyn Spectre>> {
        lock!(self.state).get_spectres()        
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
