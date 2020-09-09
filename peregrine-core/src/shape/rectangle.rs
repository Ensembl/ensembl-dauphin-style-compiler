use std::sync::Arc;
use super::core::{ AnchorPair, Patina, ShapeSet, SingleAnchor };

#[derive(Debug)]
enum RectShape {
    SingleAnchor(SingleAnchor,Patina,Vec<String>),
    DoubleAnchor(AnchorPair,Patina,Vec<String>)
}

// XXX filter allotments too!
impl RectShape {
    fn filter(self, min_value: f64, max_value: f64) -> RectShape {
        match self {
            RectShape::SingleAnchor(anchor,patina,allotment) => RectShape::SingleAnchor(anchor.filter(min_value,max_value),patina,allotment),
            RectShape::DoubleAnchor(anchor,patina,allotment) => RectShape::DoubleAnchor(anchor.filter(min_value,max_value),patina,allotment),
        }
    }
}

#[derive(Debug)]
pub struct RectangleShapeSet {
    shapes: Vec<RectShape>
}

impl RectangleShapeSet {
    pub fn new() -> RectangleShapeSet {
        RectangleShapeSet {
            shapes: vec![]
        }
    }

    pub fn add_rectangle_1(&mut self, anchors: SingleAnchor, patina: Patina, allotments: Vec<String>) {
        self.shapes.push(RectShape::SingleAnchor(anchors,patina,allotments));
    }

    pub fn add_rectangle_2(&mut self, anchors: AnchorPair, patina: Patina, allotments: Vec<String>) {
        self.shapes.push(RectShape::DoubleAnchor(anchors,patina,allotments));
    }

    pub fn filter(&mut self, min_value: f64, max_value: f64) -> RectangleShapeSet {
        let mut new = vec![];
        for shape in self.shapes.drain(..) {
            new.push(shape.filter(min_value,max_value));
        }
        RectangleShapeSet {
            shapes: new
        }
    }
}
