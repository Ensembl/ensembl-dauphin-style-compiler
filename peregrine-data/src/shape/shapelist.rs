use std::collections::HashSet;
use super::core::{ AnchorPair, Patina, SingleAnchor, Pen, Plotter };
use super::shape::Shape;
use crate::switch::allotment::{ Allotter, AllotmentHandle, AllotmentPetitioner };

#[derive(Debug)]
pub struct ShapeList {
    shapes: Vec<Shape>,
    allotments: HashSet<AllotmentHandle>
}

impl ShapeList {
    pub fn new() -> ShapeList {
        ShapeList {
            shapes: vec![],
            allotments: HashSet::new()
        }
    }

    pub fn add_allotment(&mut self, allotment: &AllotmentHandle) {
        self.allotments.insert(allotment.clone());
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

    pub fn filter(&self, min_value: f64, max_value: f64) -> ShapeList {
        let mut shapes = vec![];
        for shape in self.shapes.iter() {
            shapes.push(shape.filter(min_value,max_value));
        }
        ShapeList { shapes, allotments: self.allotments.clone() }
    }

    pub fn append(&mut self, more: &ShapeList) {
        self.shapes.extend(more.shapes.iter().cloned());
        self.allotments = self.allotments.union(&more.allotments).cloned().collect();
    }

    pub fn shapes(&self) -> &[Shape] { &self.shapes }

    pub fn make_allotter(&self, petitioner: &AllotmentPetitioner) -> Allotter {
        let handles = self.allotments.iter().cloned().collect::<Vec<_>>();
        Allotter::new(petitioner,&handles)
    }
}
