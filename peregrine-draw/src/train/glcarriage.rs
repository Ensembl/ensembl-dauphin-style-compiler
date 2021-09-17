use peregrine_data::{Assets, Carriage, CarriageId, Scale, VariableValues, ZMenuProxy};
use crate::shape::layers::drawing::{ Drawing };
use crate::webgl::DrawingSession;
use crate::webgl::global::WebGlGlobal;
use std::hash::{ Hash, Hasher };
use std::rc::Rc;
use std::sync::Mutex;
use crate::stage::stage::ReadStage;
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
    pub fn new(carriage: &Carriage, scale: &Scale, opacity: f64, gl: &mut WebGlGlobal, assets: &Assets) -> Result<GLCarriage,Message> {
        Ok(GLCarriage {
            id: carriage.id().clone(),
            opacity: Mutex::new(opacity),
            drawing: Drawing::new(Some(scale),carriage.shapes(),gl,carriage.id().left_right().0,&VariableValues::new(),assets)?
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
        self.drawing.set_zmenu_px_per_screen(stage.x().drawable_size()?);
        let opacity = self.opacity.lock().unwrap().clone();
        if self.in_view(stage)? {
            self.drawing.draw(gl,stage,session,opacity,priority)?;
        }
        Ok(())
    }

    pub(crate) fn get_hotspot(&self, stage: &ReadStage, position: (f64,f64)) -> Result<Vec<Rc<ZMenuProxy>>,Message> {
        self.drawing.get_hotspot(stage,position)
    }

    pub fn discard(&mut self, gl: &mut WebGlGlobal) -> Result<(),Message> {
        self.drawing.discard(gl)?;
        Ok(())
    }
}
