use peregrine_core::{ Carriage, CarriageId };
use crate::shape::layers::drawing::{ DrawingBuilder, Drawing };
use crate::shape::core::glshape::PreparedShape;
use crate::shape::canvas::allocator::DrawingCanvasesAllocator;
use crate::webgl::DrawingSession;
use crate::webgl::global::WebGlGlobal;
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
    pub fn new(carriage: &Carriage, opacity: f64, gl: &mut WebGlGlobal) -> anyhow::Result<GLCarriage> {
        let mut drawing = DrawingBuilder::new(gl.program_store(),carriage.id().left());
        let mut count = 0;
        let preparations : Result<Vec<PreparedShape>,_> = carriage.shapes().drain(..).map(|s| drawing.prepare_shape(s)).collect();
        let mut canvas_allocator = DrawingCanvasesAllocator::new();
        drawing.finish_preparation(gl.canvas_store_mut(), &mut canvas_allocator)?;
        let gpu_spec = gl.program_store().gpu_spec().clone();
        let canvas_builder = canvas_allocator.make_builder(gl.canvas_store_mut(),&gpu_spec)?;
        for shape in preparations?.drain(..) {
            drawing.add_shape(shape)?;
            count += 1;
        }
        console::log_1(&format!("carriage={} shape={:?}",carriage.id(),count).into());
        let drawing = drawing.build(gl.canvas_store_mut(),canvas_builder)?;
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

    pub fn discard(&mut self, gl: &mut WebGlGlobal) -> anyhow::Result<()> {
        self.drawing.discard(gl.canvas_store_mut())?;
        Ok(())
    }
}
