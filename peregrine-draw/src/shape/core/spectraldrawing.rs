use std::sync::{Arc, Mutex};

use peregrine_data::{AllAllotmentsRequest, AllotmentMetadataStore, Assets, ShapeListBuilder, VariableValues};

use crate::{Message, shape::layers::drawing::Drawing, stage::stage::ReadStage, webgl::{DrawingSession, global::WebGlGlobal}};

use super::spectre::Spectre;

fn draw_spectres(gl: &mut WebGlGlobal, assets: &Assets, allotment_petitioner: &mut AllAllotmentsRequest, allotment_metadata: &mut AllotmentMetadataStore, variables: &VariableValues<f64>, spectres: &[Spectre]) -> Result<Drawing,Message> {
    let mut shapes = ShapeListBuilder::new();
    for spectre in spectres {
        spectre.draw(&mut shapes,allotment_petitioner,allotment_metadata)?;
    }
    Drawing::new(None,shapes.build(),gl,0.,variables,assets)
}

#[derive(Clone)]
pub struct SpectralDrawing(Arc<Mutex<Option<Drawing>>>,VariableValues<f64>);

impl SpectralDrawing {
    pub fn new(variables: &VariableValues<f64>) -> SpectralDrawing {
        SpectralDrawing(Arc::new(Mutex::new(None)),variables.clone())
    }

    pub(crate) fn set(&self, gl: &mut WebGlGlobal, assets: &Assets, allotment_petitioner: &mut AllAllotmentsRequest, allotment_metadata: &mut AllotmentMetadataStore, spectres: &[Spectre]) -> Result<(),Message> {
        let mut drawing_holder = self.0.lock().unwrap();
        if let Some(drawing_holder) = drawing_holder.as_mut() {
            drawing_holder.discard(gl)?;
        }
        let mut drawing = draw_spectres(gl,assets,allotment_petitioner,allotment_metadata,&self.1,spectres)?;
        drawing.recompute()?;
        *drawing_holder = Some(drawing);
        Ok(())
    }

    pub(crate) fn variables(&self) -> &VariableValues<f64> { &self.1 }

    pub(crate) fn update(&self) -> Result<(),Message> {
        if let Some(drawing) = self.0.lock().unwrap().as_mut() {
            drawing.recompute()?;
        }
        Ok(())
    }

    pub(crate) fn draw(&mut self, gl: &mut WebGlGlobal, stage: &ReadStage, session: &DrawingSession) -> Result<(),Message> {
        if let Some(drawing) = self.0.lock().unwrap().as_mut() {
            drawing.draw(gl,stage,session,1.0,0)?;
        }
        Ok(())
    }
}
