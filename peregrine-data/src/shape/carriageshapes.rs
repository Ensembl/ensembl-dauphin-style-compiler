use std::sync::{Arc};

use crate::{LeafRequest, Shape, allotment::{core::{carriageoutput::{CarriageOutput}, leaflist::LeafList}}, ProgramShapesBuilder, ShapeRequestGroup};

pub struct CarriageShapesBuilder {
    shapes: Vec<Shape<LeafRequest>>,
    carriage_universe: Arc<LeafList>
}

impl CarriageShapesBuilder {
    pub fn from_program_shapes(input: ProgramShapesBuilder) -> CarriageShapesBuilder {
        input.to_carriage_shapes_builder()
    }

    pub(super) fn build(shapes: Vec<Shape<LeafRequest>>, universe: LeafList) -> CarriageShapesBuilder {
        CarriageShapesBuilder {
            shapes: shapes,
            carriage_universe: Arc::new(universe)
        }
    }

    pub fn empty() -> CarriageShapesBuilder {
        CarriageShapesBuilder {
            shapes: vec![],
            carriage_universe: Arc::new(LeafList::new())
        }
    }

    pub(crate) fn merge(input: Vec<Arc<CarriageShapesBuilder>>) -> CarriageShapesBuilder {
        let len : usize = input.iter().map(|x| x.shapes.len()).sum();
        let mut shapes = vec![];
        shapes.reserve(len);
        for more in &input {
            shapes.extend(more.shapes.iter().cloned());
        }
        let leafs = input.iter().map(|x| x.carriage_universe.clone()).collect::<Vec<_>>();
        CarriageShapesBuilder {
            shapes,
            carriage_universe: Arc::new(LeafList::merge(leafs))
        }
    }

    pub fn filter(&self, min_value: f64, max_value: f64) -> CarriageShapesBuilder {
        CarriageShapesBuilder {
            shapes:self.shapes.iter().map(|shape| shape.base_filter(min_value,max_value)).collect(),
            carriage_universe: self.carriage_universe.clone(),
        }
    }

    pub fn to_universe(self, extent: Option<&ShapeRequestGroup>) -> CarriageOutput {
        CarriageOutput::new(self.carriage_universe,Arc::new(self.shapes),extent)
    }
}
