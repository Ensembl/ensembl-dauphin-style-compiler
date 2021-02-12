use super::programstore::ProgramStore;
use super::layer::Layer;
use peregrine_core::Shape;
use super::super::core::glshape::add_shape_to_layer;
use crate::webgl::{ Process, DrawingSession };

pub(crate) struct Drawing {
    processes: Vec<Process>
}

pub(crate) struct DrawingBuilder {
    main_layer: Layer
}

impl DrawingBuilder {
    pub(crate) fn new(programs: &ProgramStore, left: f64) -> DrawingBuilder {
        DrawingBuilder {
            main_layer: Layer::new(programs,left)
        }
    }

    pub(crate) fn add_shape(&mut self, shape: Shape) -> anyhow::Result<()> {
        add_shape_to_layer(&mut self.main_layer, shape)
    }

    pub fn build(self) -> anyhow::Result<Drawing> {
        let mut processes = vec![];
        self.main_layer.build(&mut processes)?;
        Ok(Drawing {
            processes
        })
    }
}

impl Drawing {
    pub(crate) fn draw(&mut self, session: &DrawingSession, opacity: f64) -> anyhow::Result<()> {
        for process in &mut self.processes {
            session.run_process(process,opacity)?;
        }
        Ok(())
    }

    pub(crate) fn discard(&mut self) -> anyhow::Result<()> {
        for process in &mut self.processes {
            process.discard()?;
        }
        Ok(())
    }
}