use std::{collections::HashMap, sync::Arc};

use peregrine_toolkit::puzzle::{PuzzleBuilder, PuzzleSolution};

use crate::{allotment::{style::{pendingleaf::PendingLeaf, allotmentname::AllotmentName, holder::ContainerHolder, stylebuilder::make_transformable }, stylespec::stylegroup::AllotmentStyleGroup, boxes::{root::Root, boxtraits::Transformable}}, Pen, CarriageExtent, Shape, ShapeRequest, ShapeRequestGroup, EachOrEvery};

use super::arbitrator::BpPxConverter;

pub struct CarriageUniverseBuilder {
    leafs: HashMap<String,PendingLeaf>,
}

impl CarriageUniverseBuilder {
    pub fn new() -> CarriageUniverseBuilder {
        CarriageUniverseBuilder {
            leafs: HashMap::new()
        }
    }

    pub fn pending_leaf(&mut self, spec: &str) -> &mut PendingLeaf {
        if !self.leafs.contains_key(spec) {
            self.leafs.insert(spec.to_string(),PendingLeaf::new(&AllotmentName::new(spec)));
        }
        self.leafs.get_mut(spec).unwrap()
    }

    pub fn union(&self, other: &CarriageUniverseBuilder) -> CarriageUniverseBuilder {
        let mut leafs = self.leafs.clone();
        leafs.extend(other.leafs.iter().map(|(k,v)| (k.clone(),v.clone())));
        CarriageUniverseBuilder { leafs }
    }

    fn make_transformable(&self, extent: Option<&ShapeRequest>) {
        let builder = PuzzleBuilder::new();
        let converter = Arc::new(BpPxConverter::new2(extent));
        let root = ContainerHolder::Root(Root::new());
        make_transformable(&builder,&converter,&root,&mut self.leafs.values());
    }
}

pub struct CarriageUniverse2 {
    shapes: Arc<Vec<Shape<Arc<dyn Transformable>>>>
}

impl CarriageUniverse2 {
    pub fn new(builder: &CarriageUniverseBuilder, shapes: &[Shape<PendingLeaf>], extent: Option<&ShapeRequest>) -> CarriageUniverse2 {
        builder.make_transformable(extent);
        let shapes = shapes.iter().map(|x| 
            x.map_new_allotment(|x| x.transformable().cloned())
        ).collect::<Vec<_>>();
        CarriageUniverse2 { shapes: Arc::new(shapes) }
    }

    pub fn get(&self, solution: &PuzzleSolution) -> Vec<Shape<()>> {
        let mut out = vec![];
        for input in self.shapes.iter() {
            out.append(&mut input.map_new_allotment(|x| x.make(solution)).make(solution));
        }
        out
    }

    pub fn filter(&self, min_value: f64, max_value: f64) -> CarriageUniverse2 {
        let mut shapes = vec![];
        for shape in self.shapes.iter() {
            shapes.push(shape.filter_by_minmax(min_value,max_value));
        }
        CarriageUniverse2 { shapes: Arc::new(shapes) }
    }
}
