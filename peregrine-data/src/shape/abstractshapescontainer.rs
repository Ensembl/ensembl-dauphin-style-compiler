use std::sync::{Arc};
use crate::{ allotment::{core::{abstractcarriage::{AbstractCarriage}, leaflist::LeafList}}, ShapeRequestGroup, CarriageExtent, LoadMode};
use super::{shape::UnplacedShape, originstats::OriginStats};

pub struct FloatingShapesContainer {
    shapes: Vec<UnplacedShape>,
    carriage_universe: Arc<LeafList>,
    stats: OriginStats
}

impl FloatingShapesContainer {
    pub(super) fn build(shapes: Vec<UnplacedShape>, universe: LeafList, mode: &LoadMode) -> FloatingShapesContainer {
        FloatingShapesContainer {
            shapes: shapes,
            carriage_universe: Arc::new(universe),
            stats: OriginStats::new(mode)
        }
    }

    pub(crate) fn empty(mode: &LoadMode) -> FloatingShapesContainer {
        FloatingShapesContainer {
            shapes: vec![],
            carriage_universe: Arc::new(LeafList::new()),
            stats: OriginStats::new(mode)
        }
    }

    pub(crate) fn merge(input: Vec<Arc<FloatingShapesContainer>>) -> FloatingShapesContainer {
        let len : usize = input.iter().map(|x| x.shapes.len()).sum();
        let stats = input.iter().map(|x| &x.stats).sum();
        let mut shapes = vec![];
        shapes.reserve(len);
        for more in &input {
            shapes.extend(more.shapes.iter().cloned());
        }
        let leafs = input.iter().map(|x| x.carriage_universe.clone()).collect::<Vec<_>>();
        FloatingShapesContainer {
            shapes,
            carriage_universe: Arc::new(LeafList::merge(leafs)),
            stats
        }
    }

    pub fn filter(&self, min_value: f64, max_value: f64) -> FloatingShapesContainer {
        FloatingShapesContainer {
            shapes: self.shapes.iter().map(|shape| shape.base_filter(min_value,max_value)).collect(),
            carriage_universe: self.carriage_universe.clone(),
            stats: self.stats.clone()
        }
    }

    pub fn build_abstract_carriage(self, shape_request_group: Option<&ShapeRequestGroup>, extent: Option<&CarriageExtent>) -> AbstractCarriage {
        AbstractCarriage::new(self.carriage_universe,self.shapes,shape_request_group,extent)
    }

    pub(crate) fn stats(&self) -> &OriginStats { &self.stats }
}
