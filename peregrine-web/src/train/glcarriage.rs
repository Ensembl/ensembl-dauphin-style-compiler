use peregrine_core::{ Carriage, CarriageId };
use crate::shape::layers::drawing::{ DrawingBuilder, Drawing };
use crate::shape::core::glshape::PreparedShape;
use crate::webgl::DrawingSession;
use crate::webgl::global::WebGlGlobal;
use std::hash::{ Hash, Hasher };
use std::sync::Mutex;
use crate::shape::core::stage::ReadStage;

pub(crate) struct GLCarriage {
    id: CarriageId,
    opacity: Mutex<f64>,
    drawing: Drawing
}

impl PartialEq for GLCarriage {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for GLCarriage {}

impl Hash for GLCarriage {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.id.hash(hasher);
    }
}

impl GLCarriage {
    pub fn new(carriage: &Carriage, opacity: f64, gl: &mut WebGlGlobal) -> anyhow::Result<GLCarriage> {
        let mut drawing = DrawingBuilder::new(gl,carriage.id().left());
        let mut count = 0;
        let preparations : Result<Vec<PreparedShape>,_> = carriage.shapes().drain(..).map(|s| drawing.prepare_shape(s)).collect();
        drawing.finish_preparation(gl)?;
        for shape in preparations?.drain(..) {
            drawing.add_shape(gl,shape)?;
            count += 1;
        }
        let drawing = drawing.build(gl)?;
        Ok(GLCarriage {
            id: carriage.id().clone(),
            opacity: Mutex::new(opacity),
            drawing
        })
    }

    pub fn id(&self) -> &CarriageId { &self.id }

    pub(super) fn set_opacity(&self, amount: f64) {
        *self.opacity.lock().unwrap() = amount;
    }

    pub fn draw(&mut self, gl: &mut WebGlGlobal, stage: &ReadStage, session: &DrawingSession) -> anyhow::Result<()> {
        let opacity = self.opacity.lock().unwrap().clone();
        self.drawing.draw(gl,stage,session,opacity)
    }

    pub fn discard(&mut self, gl: &mut WebGlGlobal) -> anyhow::Result<()> {
        self.drawing.discard(gl)?;
        Ok(())
    }
}
