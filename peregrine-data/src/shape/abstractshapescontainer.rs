use std::sync::{Arc};
use crate::{ allotment::{core::{abstractcarriage::{AbstractCarriage}, leaflist::LeafList}}, ShapeRequestGroup, CarriageExtent};
use super::shape::UnplacedShape;

pub struct AbstractShapesContainer {
    shapes: Vec<UnplacedShape>,
    carriage_universe: Arc<LeafList>
}

impl AbstractShapesContainer {
    pub(super) fn build(shapes: Vec<UnplacedShape>, universe: LeafList) -> AbstractShapesContainer {
        AbstractShapesContainer {
            shapes: shapes,
            carriage_universe: Arc::new(universe)
        }
    }

    pub fn empty() -> AbstractShapesContainer {
        AbstractShapesContainer {
            shapes: vec![],
            carriage_universe: Arc::new(LeafList::new())
        }
    }

    pub(crate) fn merge(input: Vec<Arc<AbstractShapesContainer>>) -> AbstractShapesContainer {
        let len : usize = input.iter().map(|x| x.shapes.len()).sum();
        let mut shapes = vec![];
        shapes.reserve(len);
        for more in &input {
            shapes.extend(more.shapes.iter().cloned());
        }
        let leafs = input.iter().map(|x| x.carriage_universe.clone()).collect::<Vec<_>>();
        AbstractShapesContainer {
            shapes,
            carriage_universe: Arc::new(LeafList::merge(leafs))
        }
    }

    pub fn filter(&self, min_value: f64, max_value: f64) -> AbstractShapesContainer {
        AbstractShapesContainer {
            shapes:self.shapes.iter().map(|shape| shape.base_filter(min_value,max_value)).collect(),
            carriage_universe: self.carriage_universe.clone(),
        }
    }

    pub fn build_abstract_carriage(self, shape_request_group: Option<&ShapeRequestGroup>, extent: Option<&CarriageExtent>) -> AbstractCarriage {
        AbstractCarriage::new(self.carriage_universe,self.shapes,shape_request_group,extent)
    }
}
