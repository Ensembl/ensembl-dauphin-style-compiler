use super::programstore::ProgramStore;
use super::layer::Layer;
use peregrine_core::Shape;
use super::super::core::glshape::add_shape_to_layer;
use crate::webgl::Process;

pub(crate) struct Drawing {
    processes: Vec<Process>
}

pub(crate) struct DrawingBuilder {
    main_layer: Layer
}

impl DrawingBuilder {
    pub(crate) fn new(programs: &ProgramStore) -> DrawingBuilder {
        DrawingBuilder {
            main_layer: Layer::new(programs)
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
    pub(crate) fn draw(&self) -> anyhow::Result<()> {
        for process in &self.processes {
            process.draw()?;
        }
        Ok(())
    }
}