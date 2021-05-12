use std::sync::Arc;
use std::collections::HashSet;
use super::core::{ AnchorPair, Patina, SingleAnchor, Pen, Plotter };
use crate::{ Shape, SpaceBase, SpaceBaseArea };
use crate::switch::allotment::{ Allotter, AllotmentHandle, AllotmentPetitioner };

#[derive(Debug)]
pub struct ShapeListBuilder {
    shapes: Vec<Shape>,
    allotments: HashSet<AllotmentHandle>
}

impl ShapeListBuilder {
    pub fn new() -> ShapeListBuilder {
        ShapeListBuilder {
            shapes: vec![],
            allotments: HashSet::new()
        }
    }

    pub fn len(&self) -> usize { self.shapes.len() }

    pub fn add_allotment(&mut self, allotment: &AllotmentHandle) {
        self.allotments.insert(allotment.clone());
    }

    pub fn add_rectangle(&mut self, top_left: SpaceBase<AllotmentHandle>, bottom_right: SpaceBase<AllotmentHandle>, patina: Patina) {
        self.shapes.push(Shape::SpaceBaseRect(SpaceBaseArea::new(top_left,bottom_right),patina));
    }

    pub fn add_rectangle_1(&mut self, anchors: SingleAnchor, patina: Patina, allotments: Vec<AllotmentHandle>, x_size: Vec<f64>, y_size: Vec<f64>) {
        self.shapes.push(Shape::SingleAnchorRect(anchors,patina,allotments,x_size,y_size));
    }

    pub fn add_rectangle_2(&mut self, anchors: AnchorPair, patina: Patina, allotments: Vec<AllotmentHandle>) {
        self.shapes.push(Shape::DoubleAnchorRect(anchors,patina,allotments));
    }

    pub fn add_text(&mut self, anchors: SingleAnchor, pen: Pen, text: Vec<String>, allotments: Vec<AllotmentHandle>) {
        self.shapes.push(Shape::Text(anchors,pen,text,allotments));
    }

    pub fn add_wiggle(&mut self, min: f64, max: f64, plotter: Plotter, values: Vec<Option<f64>>, allotment: AllotmentHandle) {
        self.shapes.push(Shape::Wiggle((min,max),values,plotter,allotment))
    }

    pub fn filter(&self, min_value: f64, max_value: f64) -> ShapeListBuilder {
        let mut shapes = vec![];
        for shape in self.shapes.iter() {
            shapes.push(shape.filter(min_value,max_value));
        }
        ShapeListBuilder { shapes, allotments: self.allotments.clone() }
    }

    pub fn append(&mut self, more: &ShapeListBuilder) {
        self.shapes.extend(more.shapes.iter().cloned());
        self.allotments = self.allotments.union(&more.allotments).cloned().collect();
    }

    pub fn build(self, petitioner: &AllotmentPetitioner) -> ShapeList {
        ShapeList::new(self,petitioner)
    }
}

#[derive(Clone)]
pub struct ShapeList {
    shapes: Arc<Vec<Shape>>,
    allotter: Arc<Allotter>
}

impl ShapeList {
    pub fn empty() -> ShapeList {
        ShapeList {
            shapes: Arc::new(vec![]),
            allotter: Arc::new(Allotter::empty())
        }
    }

    fn new(builder: ShapeListBuilder, petitioner: &AllotmentPetitioner) -> ShapeList {
        let handles = builder.allotments.iter().cloned().collect::<Vec<_>>();
        ShapeList {
            shapes: Arc::new(builder.shapes),
            allotter: Arc::new(Allotter::new(petitioner,&handles))
        }
    }

    pub fn shapes(&self) -> Arc<Vec<Shape>> { self.shapes.clone() }
    pub fn allotter(&self) -> Arc<Allotter> { self.allotter.clone() }
}
