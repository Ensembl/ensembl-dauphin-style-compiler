use std::sync::{Arc, Mutex};
use peregrine_data::{Assets, reactive::Reactive, ProgramShapesBuilder, DataMessage };
use peregrine_toolkit::{lock, puzzle::AnswerAllocator};
use peregrine_toolkit_async::{sync::retainer::{RetainTest, Retainer, retainer}};
use crate::{Message, shape::{layers::drawing::Drawing}, stage::stage::ReadStage, webgl::{DrawingSession, global::WebGlGlobal}, PgCommanderWeb};

use super::{spectre::{Spectre}};

async fn draw_spectres<X>(gl: &Arc<Mutex<WebGlGlobal>>, assets: &Assets, spectres: &[X], retain_test: &RetainTest) -> Result<Drawing,Message> where X: Spectre {
    let mut shapes = ProgramShapesBuilder::new(&Assets::empty(),&peregrine_data::LoadMode::RealTime);
    for spectre in spectres {
        spectre.draw(&mut shapes)?;
    }
    let raw = shapes.to_abstract_shapes_container();
    let list = raw.build_abstract_carriage(None,None);
    let mut aia = AnswerAllocator::new();
    let shapes = list.unfloat_shapes(&mut aia.get());
    let shapes = shapes.map_err(|e| Message::DataError(peregrine_data::DataMessage::XXXTransitional(e)))?;
    Drawing::new(None,Arc::new(shapes),gl,0.,assets,retain_test).await.transpose().unwrap().map_err(|e| Message::DataError(DataMessage::XXXTransitional(e) ))
}

async fn draw<X>(gl: &Arc<Mutex<WebGlGlobal>>, assets: &Assets, spectres: &[X]) -> Result<(Drawing,Retainer),Message> where X: Spectre {
    let (retainer,retain_test) = retainer();
    let mut drawing = draw_spectres(gl,assets,spectres,&retain_test).await?;
    drawing.recompute(&*lock!(gl)).map_err(|e| Message::DataError(DataMessage::XXXTransitional(e) ))?;
    Ok((drawing,retainer))
}

#[derive(Clone)]
pub struct SpectralDrawing {
    commander: PgCommanderWeb,
    drawing: Arc<Mutex<Option<(Drawing,Retainer)>>>,
    index: Arc<Mutex<u64>>,
    reactive: Reactive<'static>
}

impl SpectralDrawing {
    pub fn new(commander: &PgCommanderWeb, reactive: &Reactive<'static>) -> SpectralDrawing {
        SpectralDrawing {
            commander: commander.clone(),
            drawing: Arc::new(Mutex::new(None)),
            index: Arc::new(Mutex::new(0)),
            reactive: reactive.clone()
        }
    }

    pub(crate) fn set<X>(&mut self, gl: &Arc<Mutex<WebGlGlobal>>, assets: &Assets, spectres: &[X]) where X: Spectre + Clone + 'static {
        let mut index = lock!(self.index);
        *index += 1;
        let our_index = *index;
        drop(index);
        let gl = gl.clone();
        let assets = assets.clone();
        let spectres = spectres.to_vec();
        let self2 = self.clone();
        self.commander.add::<()>("load", 0, None, None, Box::pin(async move {
            if let Ok((drawing,retainer)) = draw(&gl,&assets,&spectres).await {
                let index = lock!(self2.index);
                if *index == our_index {
                    *lock!(self2.drawing) = Some((drawing,retainer));
                }
                drop(index);
            }
            Ok(())
        }));
    }

    pub(crate) fn reactive(&self) -> &Reactive<'static> { &self.reactive }

    pub(crate) fn update(&self, gl: &WebGlGlobal) -> Result<(),Message> {
        if let Some((drawing,_)) = lock!(self.drawing).as_mut() {
            drawing.recompute(gl).map_err(|e| Message::DataError(DataMessage::XXXTransitional(e) ))?;
        }
        self.reactive.run_observers();
        Ok(())
    }

    pub(crate) fn draw(&mut self, gl: &mut WebGlGlobal, stage: &ReadStage, session: &mut DrawingSession) -> Result<(),Message> {
        if let Some((drawing,_)) = lock!(self.drawing).as_mut() {
            drawing.draw(gl,stage,session,1.0).map_err(|e| Message::DataError(DataMessage::XXXTransitional(e) ))?;
        }
        Ok(())
    }
}
