use super::core::{ AnchorPair, Patina, SingleAnchor, filter, bulk, Pen, Plotter };
use std::cmp::{ max, min };
use super::shape::Shape;

#[derive(Debug)]
pub struct ShapeList {
    shapes: Vec<Shape>
}

impl ShapeList {
    pub fn new() -> ShapeList {
        ShapeList {
            shapes: vec![]
        }
    }

    pub fn add_rectangle_1(&mut self, anchors: SingleAnchor, patina: Patina, allotments: Vec<String>, x_size: Vec<f64>, y_size: Vec<f64>) {
        self.shapes.push(Shape::SingleAnchorRect(anchors,patina,allotments,x_size,y_size));
    }

    pub fn add_rectangle_2(&mut self, anchors: AnchorPair, patina: Patina, allotments: Vec<String>) {
        self.shapes.push(Shape::DoubleAnchorRect(anchors,patina,allotments));
    }

    pub fn add_text(&mut self, anchors: SingleAnchor, pen: Pen, text: Vec<String>, allotments: Vec<String>) {
        self.shapes.push(Shape::Text(anchors,pen,text,allotments));
    }

    pub fn add_wiggle(&mut self, min: f64, max: f64, plotter: Plotter, values: Vec<Option<f64>>, allotment: String) {
        self.shapes.push(Shape::Wiggle((min,max),values,plotter,allotment))
    }

    pub fn filter(&self, min_value: f64, max_value: f64) -> ShapeList {
        let mut new = vec![];
        for shape in self.shapes.iter() {
            new.push(shape.filter(min_value,max_value));
        }
        ShapeList {
            shapes: new
        }
    }

    pub fn append(&mut self, more: &ShapeList) {
        self.shapes.extend(more.shapes.iter().cloned());
    }
}
