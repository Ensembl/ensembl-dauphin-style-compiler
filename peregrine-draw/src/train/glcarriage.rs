use peregrine_data::{ Carriage, CarriageId };
use crate::shape::layers::drawing::{ DrawingBuilder, Drawing };
use crate::shape::core::glshape::PreparedShape;
use crate::webgl::DrawingSession;
use crate::webgl::global::WebGlGlobal;
use std::hash::{ Hash, Hasher };
use std::sync::Mutex;
use crate::shape::core::stage::ReadStage;
use crate::shape::layers::drawingzmenus::ZMenuEvent;
use crate::util::message::Message;

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
    pub fn new(carriage: &Carriage, opacity: f64, gl: &mut WebGlGlobal) -> Result<GLCarriage,Message> {
        let mut drawing = DrawingBuilder::new(gl,carriage.id().left());
        let preparations : Result<Vec<PreparedShape>,_> = carriage.shapes().drain(..).map(|s| drawing.prepare_shape(s)).collect();
        drawing.finish_preparation(gl)?;
        for shape in preparations?.drain(..) {
            drawing.add_shape(gl,shape)?;
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

    pub fn draw(&mut self, gl: &mut WebGlGlobal, stage: &ReadStage, session: &DrawingSession) ->Result<(),Message> {
        let opacity = self.opacity.lock().unwrap().clone();
        self.drawing.draw(gl,stage,session,opacity)
    }

    pub(crate) fn intersects(&self, stage: &ReadStage, mouse: (u32,u32)) -> Result<Option<ZMenuEvent>,Message> {
        self.drawing.intersects(stage,mouse)
    }

    pub(crate) fn intersects_fast(&self, stage: &ReadStage, mouse: (u32,u32)) -> Result<bool,Message> {
        self.drawing.intersects_fast(stage,mouse)
    }

    pub fn discard(&mut self, gl: &mut WebGlGlobal) -> Result<(),Message> {
        self.drawing.discard(gl)?;
        Ok(())
    }
}
