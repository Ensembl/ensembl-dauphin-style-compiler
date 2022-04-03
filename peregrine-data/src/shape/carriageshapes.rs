use std::sync::Arc;

use crate::{LeafRequest, Shape, allotment::{core::{carriageoutput::{CarriageOutput}, leaflist::LeafList}, stylespec::{stylegroup::AllotmentStyleGroup, styletree::StyleTree}}, ProgramShapesBuilder, DataMessage, ShapeRequestGroup};

#[derive(Clone)]
pub struct CarriageShapesBuilder {
    shapes: Arc<Vec<Shape<LeafRequest>>>,
    carriage_universe: Arc<LeafList>
}

impl CarriageShapesBuilder {
    pub fn from_program_shapes(input: ProgramShapesBuilder) -> CarriageShapesBuilder {
        input.to_carriage_shapes_builder()
    }

    pub(super) fn build(shapes: Vec<Shape<LeafRequest>>, universe: LeafList) -> CarriageShapesBuilder {
        CarriageShapesBuilder {
            shapes: Arc::new(shapes),
            carriage_universe: Arc::new(universe)
        }
    }

    pub fn empty() -> CarriageShapesBuilder {
        CarriageShapesBuilder {
            shapes: Arc::new(vec![]),
            carriage_universe: Arc::new(LeafList::new())
        }
    }

    pub fn union(&self, more: &CarriageShapesBuilder) -> CarriageShapesBuilder {
        let mut shapes = self.shapes.as_ref().to_vec();
        shapes.extend(more.shapes.iter().cloned());
        CarriageShapesBuilder {
            shapes: Arc::new(shapes),
            carriage_universe: Arc::new(self.carriage_universe.union(&more.carriage_universe))
        }
    }

    pub fn filter(&self, min_value: f64, max_value: f64) -> CarriageShapesBuilder {
        CarriageShapesBuilder {
            shapes: Arc::new(self.shapes.iter().map(|shape| shape.base_filter(min_value,max_value)).collect()),
            carriage_universe: self.carriage_universe.clone(),
        }
    }

    pub fn to_universe(&self, extent: Option<&ShapeRequestGroup>) -> Result<CarriageOutput,DataMessage> {
        CarriageOutput::new(&self.carriage_universe,&self.shapes,extent)
    }
}
