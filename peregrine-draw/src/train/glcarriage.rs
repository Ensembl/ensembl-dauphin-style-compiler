use peregrine_data::{Carriage, CarriageId, VariableValues};
use crate::shape::layers::drawing::{ Drawing };
use crate::webgl::DrawingSession;
use crate::webgl::global::WebGlGlobal;
use std::hash::{ Hash, Hasher };
use std::sync::Mutex;
use crate::stage::stage::ReadStage;
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
        Ok(GLCarriage {
            id: carriage.id().clone(),
            opacity: Mutex::new(opacity),
            drawing: Drawing::new(carriage.shapes(),gl,carriage.id().left_right().0,&VariableValues::new())?
        })
    }

    pub fn id(&self) -> &CarriageId { &self.id }

    pub(super) fn set_opacity(&self, amount: f64) {
        *self.opacity.lock().unwrap() = amount;
    }

    pub fn priority_range(&self) -> (i8,i8) { self.drawing.priority_range() }

    fn in_view(&self, stage: &ReadStage) -> Result<bool,Message> {
        let stage = stage.x().left_right()?;
        let carriage = self.id.left_right();
        Ok(!(stage.0 > carriage.1 || stage.1 < carriage.0))
    }

    pub fn draw(&mut self, gl: &mut WebGlGlobal, stage: &ReadStage, session: &DrawingSession, priority: i8) ->Result<(),Message> {
        let opacity = self.opacity.lock().unwrap().clone();
        if self.in_view(stage)? {
            self.drawing.draw(gl,stage,session,opacity,priority)?;
        }
        Ok(())
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
