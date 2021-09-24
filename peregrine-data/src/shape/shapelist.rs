use std::sync::Arc;
use std::collections::HashSet;
use super::core::{ Patina, Pen, Plotter };
use crate::{AllotmentMetadataStore, DataFilter, HoleySpaceBase, HoleySpaceBaseArea, Shape, Universe, allotment::allotmentrequest::AllotmentRequest};

pub struct ShapeListBuilder {
    shapes: Vec<Shape>,
    allotments: HashSet<AllotmentRequest>,
    universe: Universe
}

impl ShapeListBuilder {
    pub fn new(allotment_metadata: &AllotmentMetadataStore) -> ShapeListBuilder {
        ShapeListBuilder {
            shapes: vec![],
            allotments: HashSet::new(),
            universe: Universe::new(allotment_metadata)
        }
    }

    pub fn universe(&self) -> &Universe { &self.universe }

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

    pub fn use_allotment(&mut self, allotment: &AllotmentRequest) {
        if !allotment.is_dustbin() {
            self.allotments.insert(allotment.clone());
        }
    }
    
    pub fn add_rectangle(&mut self, area: HoleySpaceBaseArea, patina: Patina, allotments: Vec<AllotmentRequest>) {
        for (group,mut filter) in DataFilter::demerge(&allotments, |x| { x.allotment_group() }) {
            filter.set_size(area.len());
            self.push(Shape::SpaceBaseRect(area.filter(&filter),patina.clone(),filter.filter(&allotments),group));
        }
    }

    pub fn add_text(&mut self, position: HoleySpaceBase, pen: Pen, text: Vec<String>, allotments: Vec<AllotmentRequest>) {
        for (group,mut filter) in DataFilter::demerge(&allotments, |x| { x.allotment_group() }) {
            filter.set_size(position.len());
            self.push(Shape::Text(position.filter(&filter),pen.filter(&filter),filter.filter(&text),filter.filter(&allotments),group));
        }
    }

    pub fn add_image(&mut self, position: HoleySpaceBase, depth: i8,images: Vec<String>, allotments: Vec<AllotmentRequest>) {
        for (group,mut filter) in DataFilter::demerge(&allotments, |x| { x.allotment_group() }) {
            filter.set_size(position.len());
            self.push(Shape::Image(position.filter(&filter),depth,filter.filter(&images),filter.filter(&allotments),group));
        }
    }

    pub fn add_wiggle(&mut self, min: f64, max: f64, plotter: Plotter, values: Vec<Option<f64>>, allotment: AllotmentRequest) {
        self.push(Shape::Wiggle((min,max),values,plotter,allotment))
    }

    pub fn filter(&self, min_value: f64, max_value: f64) -> ShapeListBuilder {
        let mut shapes = vec![];
        for shape in self.shapes.iter() {
            shapes.push(shape.filter(min_value,max_value));
        }
        ShapeListBuilder { shapes, allotments: self.allotments.clone(), universe: self.universe.clone() }
    }

    pub fn append(&mut self, more: &ShapeListBuilder) {
        self.shapes.extend(more.shapes.iter().cloned());
        self.allotments = self.allotments.union(&more.allotments).cloned().collect();
        self.universe.union(&more.universe);
    }

    pub fn build(self) -> ShapeList {
        ShapeList::new(self)
    }
}

#[derive(Clone)]
pub struct ShapeList {
    shapes: Arc<Vec<Shape>>,
    universe: Universe
}

impl ShapeList {
    pub fn empty() -> ShapeList {
        ShapeList {
            shapes: Arc::new(vec![]),
            universe: Universe::new(&AllotmentMetadataStore::new())
        }
    }

    fn new(builder: ShapeListBuilder) -> ShapeList {
        builder.universe.allot();
        ShapeList {
            universe: builder.universe.clone(),
            shapes: Arc::new(builder.shapes)
        }
    }

    pub fn universe(&self) -> &Universe { &self.universe }
    pub fn len(&self) -> usize { self.shapes.len() }
    pub fn shapes(&self) -> Arc<Vec<Shape>> { self.shapes.clone() }
}
