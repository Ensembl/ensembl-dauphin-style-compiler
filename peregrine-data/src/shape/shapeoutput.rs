use crate::lock;
use std::sync::{ Arc, Mutex };
use super::shapelist::ShapeList;
use owning_ref::MutexGuardRefMut;
use super::core::{ Patina, AnchorPair, SingleAnchor, Pen, Plotter };
use crate::Track;

#[derive(Debug)]
pub struct ShapeOutputData {
    shapes: ShapeList
}

impl ShapeOutputData {
    fn new() -> ShapeOutputData {
        ShapeOutputData {
            shapes: ShapeList::new()
        }
    }

    fn track(&mut self) -> &mut ShapeList {
        &mut self.shapes
    }

    fn filter(&self, min_value: f64, max_value: f64) -> ShapeOutputData {
        ShapeOutputData {
            shapes: self.shapes.filter(min_value,max_value)
        }
    }
}

#[derive(Clone,Debug)]
pub struct ShapeOutput {
    data: Arc<Mutex<ShapeOutputData>>
}

impl ShapeOutput {
    pub fn new(track: &Track) -> ShapeOutput {
        ShapeOutput {
            data: Arc::new(Mutex::new(ShapeOutputData::new()))
        }
    }

    pub fn track_shapes(&self) -> MutexGuardRefMut<ShapeOutputData,ShapeList> {
        MutexGuardRefMut::new(self.data.lock().unwrap()).map_mut(|x| x.track())
    }

    pub fn filter(&self, min_value: f64, max_value: f64) -> ShapeOutput {
        let data = lock!(self.data).filter(min_value,max_value);
        ShapeOutput {
            data: Arc::new(Mutex::new(data))
        }
    }

    pub fn add_rectangle_1(&self, anchors: SingleAnchor, x_size: Vec<f64>, y_size: Vec<f64>, patina: Patina, allotments: Vec<String>, tracks: Vec<Track>) {
        self.track_shapes().add_rectangle_1(anchors,patina,allotments,x_size,y_size);
    }

    pub fn add_rectangle_2(&self, anchors: AnchorPair, patina: Patina, allotments: Vec<String>, tracks: Vec<Track>) {
        self.track_shapes().add_rectangle_2(anchors,patina,allotments);
    }

    pub fn add_text(&self, anchors: SingleAnchor, pen: Pen, text: Vec<String>, allotments: Vec<String>, tracks: Vec<Track>) {
        self.track_shapes().add_text(anchors,pen,text,allotments);
    }

    pub fn add_wiggle(&self, min: f64, max: f64, plotter: Plotter, values: Vec<Option<f64>>, allotment: String) {
        self.track_shapes().add_wiggle(min,max,plotter,values,allotment);
    }
}
