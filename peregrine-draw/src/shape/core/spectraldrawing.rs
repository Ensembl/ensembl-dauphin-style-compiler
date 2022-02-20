use std::sync::{Arc, Mutex};
use peregrine_data::{AllotmentMetadataStore, Assets, ShapeListBuilder, CarriageShapeList, reactive::Reactive};
use peregrine_toolkit::lock;
use crate::{Message, shape::layers::drawing::Drawing, stage::stage::ReadStage, webgl::{DrawingSession, global::WebGlGlobal}};
use super::spectre::Spectre;

fn draw_spectres(gl: &Arc<Mutex<WebGlGlobal>>, assets: &Assets, allotment_metadata: &AllotmentMetadataStore, spectres: &[Spectre]) -> Result<Drawing,Message> {
    let mut shapes = ShapeListBuilder::new(&allotment_metadata,&Assets::empty());
    for spectre in spectres {
        spectre.draw(&mut shapes,allotment_metadata)?;
    }
    let shape_list = CarriageShapeList::new(shapes,None).map_err(|e| Message::DataError(e))?;
    Drawing::new_sync(None,shape_list,gl,0.,assets)
}

#[derive(Clone)]
pub struct SpectralDrawing(Arc<Mutex<Option<Drawing>>>,Reactive<'static>);

impl SpectralDrawing {
    pub fn new(reactive: &Reactive<'static>) -> SpectralDrawing {
        SpectralDrawing(Arc::new(Mutex::new(None)),reactive.clone())
    }

    pub(crate) fn set(&self, gl: &Arc<Mutex<WebGlGlobal>>, assets: &Assets, allotment_metadata: &AllotmentMetadataStore, spectres: &[Spectre]) -> Result<(),Message> {
        let mut drawing_holder = self.0.lock().unwrap();
        if let Some(drawing_holder) = drawing_holder.as_mut() {
            drawing_holder.discard(&mut *lock!(gl))?;
        }
        let mut drawing = draw_spectres(gl,assets,allotment_metadata,spectres)?;
        drawing.recompute()?;
        *drawing_holder = Some(drawing);
        Ok(())
    }

    pub(crate) fn reactive(&self) -> &Reactive<'static> { &self.1 }

    pub(crate) fn update(&self) -> Result<(),Message> {
        if let Some(drawing) = self.0.lock().unwrap().as_mut() {
            drawing.recompute()?;
        }
        self.1.run_observers();
        Ok(())
    }

    pub(crate) fn draw(&mut self, gl: &mut WebGlGlobal, stage: &ReadStage, session: &mut DrawingSession) -> Result<(),Message> {
        if let Some(drawing) = self.0.lock().unwrap().as_mut() {
            drawing.draw(gl,stage,session,1.0)?;
        }
        Ok(())
    }
}
