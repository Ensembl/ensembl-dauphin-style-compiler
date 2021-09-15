use std::sync::Arc;
use std::collections::HashSet;
use super::core::{ Patina, Pen, Plotter };
use crate::{HoleySpaceBase, HoleySpaceBaseArea, Shape };
use crate::switch::allotter::{ Allotter };
use crate::switch::allotment::{ AllotmentHandle, AllotmentPetitioner };

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

    fn push(&mut self, shape: Shape) {
        let shape =shape.remove_nulls();
        if !shape.is_empty() {
            self.shapes.push(shape);
        }
    }

    pub fn len(&self) -> usize { self.shapes.len() }
    pub fn vec_len(&self) -> usize {
        let mut out = 0;
        for shape in &self.shapes {
            out += shape.len();
        }
        out
    }

    pub fn add_allotment(&mut self, allotment: &AllotmentHandle) {
        if !allotment.is_null() {
            self.allotments.insert(allotment.clone());
        }
    }

    pub fn add_rectangle(&mut self, area: HoleySpaceBaseArea, patina: Patina, allotments: Vec<AllotmentHandle>) {
        self.push(Shape::SpaceBaseRect(area,patina,allotments));
    }

    pub fn add_text(&mut self, position: HoleySpaceBase, pen: Pen, text: Vec<String>, allotments: Vec<AllotmentHandle>) {
        self.push(Shape::Text(position,pen,text,allotments));
    }

    pub fn add_image(&mut self, position: HoleySpaceBase, images: Vec<String>, allotments: Vec<AllotmentHandle>) {
        self.push(Shape::Image(position,images,allotments));
    }

    pub fn add_wiggle(&mut self, min: f64, max: f64, plotter: Plotter, values: Vec<Option<f64>>, allotment: AllotmentHandle) {
        self.push(Shape::Wiggle((min,max),values,plotter,allotment))
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

    pub fn len(&self) -> usize { self.shapes.len() }
    pub fn shapes(&self) -> Arc<Vec<Shape>> { self.shapes.clone() }
    pub fn allotter(&self) -> Arc<Allotter> { self.allotter.clone() }
}
