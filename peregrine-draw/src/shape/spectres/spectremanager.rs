use std::sync::{ Arc, Mutex, Weak };
use keyed::{ keyed_handle };
use peregrine_data::{Assets, reactive::Reactive};
use peregrine_toolkit_async::{sync::needed::{Needed, NeededLock, NeededOnDrop}};
use peregrine_toolkit::{lock};
use crate::{Message, run::PgPeregrineConfig, stage::stage::ReadStage, webgl::{DrawingSession, global::WebGlGlobal}, PgCommanderWeb, shape::spectres::{ants::{MarchingAnts}, stain::Stain}};
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

keyed_handle!(SpectreId);

struct SpectreState {
    spectres: Vec<(Weak<dyn Spectre>,NeededOnDrop)>,
    new_shapes: Needed,
    redraw_needed: Needed,
    redraw_lock: Option<NeededLock>
}

impl SpectreState {
    pub(crate) fn new(redraw_needed: &Needed) -> SpectreState {
        SpectreState {
            spectres: vec![],
            new_shapes: Needed::new(),
            redraw_needed: redraw_needed.clone(),
            redraw_lock: None
        }
    }

    fn add(&mut self, spectre: Arc<dyn Spectre>) {
        self.new_shapes.set();
        if self.redraw_lock.is_none() {
            self.redraw_lock = Some(self.redraw_needed.lock());
        }
        self.spectres.push((Arc::downgrade(&spectre),self.new_shapes.needed_on_drop()));
    }

    fn get_spectres(&mut self) -> Vec<Arc<dyn Spectre>> {
        let mut out = vec![];
        let mut new = vec![];
        for (spectre,dropper) in self.spectres.drain(..) {
            if let Some(spectre) = spectre.upgrade() {
                new.push((Arc::downgrade(&spectre),dropper));
                out.push(spectre);
            }
        }
        self.spectres = new;
        if out.len() == 0 {
            self.redraw_lock = None;
        }
        out
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

    pub(crate) fn marching_ants(&mut self) -> Result<Arc<MarchingAnts>,Message> {
        MarchingAnts::new(&self.config,&self)
    }

    pub(crate) fn stain(&mut self, flip: bool) -> Result<Arc<Stain>,Message> {
        Stain::new(&self.config,&self,flip)
    }

    pub(crate) fn add<X>(&self, spectre: &Arc<X>) where X: Spectre + 'static {
        lock!(self.state).add(spectre.clone());
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