use peregrine_core::{ Carriage, CarriageId };
use crate::shape::layers::programstore::ProgramStore;
use crate::shape::layers::drawing::{ DrawingBuilder, Drawing };
use crate::webgl::DrawingSession;
use std::hash::{ Hash, Hasher };
use std::sync::Mutex;
use web_sys::console;

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
    pub fn new(carriage: &Carriage, opacity: f64, programs: &ProgramStore) -> anyhow::Result<GLCarriage> {
        let mut drawing = DrawingBuilder::new(programs,carriage.id().left());
        let mut count = 0;
        for shape in carriage.shapes().drain(..) {
            drawing.add_shape(shape)?;
            count += 1;
        }
        console::log_1(&format!("carriage={} shape={:?}",carriage.id(),count).into());
        let drawing = drawing.build()?;
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

    pub fn draw(&mut self, session: &DrawingSession) -> anyhow::Result<()> {
        let opacity = self.opacity.lock().unwrap().clone();
        self.drawing.draw(session,opacity)
    }

    pub fn discard(&mut self) -> anyhow::Result<()> {
        self.drawing.discard()
    }
}
