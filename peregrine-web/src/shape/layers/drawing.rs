use super::programstore::ProgramStore;
use super::layer::Layer;
use peregrine_core::Shape;
use super::super::core::glshape::add_shape_to_layer;

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
}
