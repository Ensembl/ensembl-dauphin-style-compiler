use std::sync::{Arc, Mutex};

use peregrine_data::{AllotmentPetitioner, ShapeListBuilder};

use crate::{Message, shape::layers::drawing::Drawing, stage::stage::ReadStage, webgl::{DrawingSession, global::WebGlGlobal}};

use super::spectre::Spectre;

fn draw_spectres(gl: &mut WebGlGlobal, allotment_petitioner: &mut AllotmentPetitioner, spectres: &[Spectre]) -> Result<Drawing,Message> {
    let mut shapes = ShapeListBuilder::new();
    for spectre in spectres {
        spectre.draw(&mut shapes,allotment_petitioner)?;
    }
    Drawing::new(shapes.build(allotment_petitioner),gl,0.)
}

#[derive(Clone)]
pub struct SpectralDrawing(Arc<Mutex<Option<Drawing>>>);

impl SpectralDrawing {
    pub fn new() -> SpectralDrawing {
        SpectralDrawing(Arc::new(Mutex::new(None)))
    }

    pub(crate) fn update(&self, gl: &mut WebGlGlobal, allotment_petitioner: &mut AllotmentPetitioner, spectres: &[Spectre]) -> Result<(),Message> {
        *self.0.lock().unwrap() = Some(draw_spectres(gl,allotment_petitioner,spectres)?);
        Ok(())
    }

    pub(crate) fn draw(&mut self, gl: &mut WebGlGlobal, stage: &ReadStage, session: &DrawingSession) -> Result<(),Message> {
        if let Some(drawing) = self.0.lock().unwrap().as_mut() {
            drawing.draw(gl,stage,session,1.0)?;
        }
        Ok(())
    }
}