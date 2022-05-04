use std::sync::{Arc, Mutex};
use peregrine_data::{Assets, reactive::Reactive, ProgramShapesBuilder, CarriageShapesBuilder};
use peregrine_toolkit::{lock, puzzle::AnswerAllocator, sync::retainer::{RetainTest, Retainer, retainer}};
use crate::{Message, shape::layers::drawing::Drawing, stage::stage::ReadStage, webgl::{DrawingSession, global::WebGlGlobal}, PgCommanderWeb};
use super::spectre::Spectre;

async fn draw_spectres(gl: &Arc<Mutex<WebGlGlobal>>, assets: &Assets, spectres: &[Spectre], retain_test: &RetainTest) -> Result<Drawing,Message> {
    let mut shapes = ProgramShapesBuilder::new(&Assets::empty());
    for spectre in spectres {
        spectre.draw(&mut shapes)?;
    }
    let raw = CarriageShapesBuilder::from_program_shapes(shapes);
    let list = raw.to_universe(None);
    let mut aia = AnswerAllocator::new();
    let shapes = list.make(&mut aia.get());
    let shapes = shapes.map_err(|e| Message::DataError(e))?;
    Drawing::new(None,Arc::new(shapes),gl,0.,assets,retain_test).await.transpose().unwrap()
}

async fn draw(gl: &Arc<Mutex<WebGlGlobal>>, assets: &Assets, spectres: &[Spectre]) -> Result<(Drawing,Retainer),Message> {
    let (retainer,retain_test) = retainer();
    let mut drawing = draw_spectres(gl,assets,spectres,&retain_test).await?;
    drawing.recompute()?;
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

    pub(crate) fn set(&mut self, gl: &Arc<Mutex<WebGlGlobal>>, assets: &Assets, spectres: &[Spectre]) {
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
                    let mut drawing_holder = lock!(self2.drawing);
                    if let Some((mut drawing,_)) = drawing_holder.take() {
                        drawing.discard(&mut *lock!(gl));
                    }
                    *drawing_holder = Some((drawing,retainer));
                }
                drop(index);
            }
            Ok(())
        }));
    }

    pub(crate) fn reactive(&self) -> &Reactive<'static> { &self.reactive }

    pub(crate) fn update(&self) -> Result<(),Message> {
        if let Some((drawing,_)) = lock!(self.drawing).as_mut() {
            drawing.recompute()?;
        }
        self.reactive.run_observers();
        Ok(())
    }

    pub(crate) fn draw(&mut self, gl: &mut WebGlGlobal, stage: &ReadStage, session: &mut DrawingSession) -> Result<(),Message> {
        if let Some((drawing,_)) = lock!(self.drawing).as_mut() {
            drawing.draw(gl,stage,session,1.0)?;
        }
        Ok(())
    }
}
