use crate::lock;
use std::sync::{ Arc, Mutex };
use super::rectangle::RectangleShapeSet;
use owning_ref::MutexGuardRefMut;
use super::core::ShapeSet;

#[derive(Debug)]
pub struct ShapeZooData {
    rectangle: RectangleShapeSet
}

impl ShapeZooData {
    fn new() -> ShapeZooData {
        ShapeZooData {
            rectangle: RectangleShapeSet::new()
        }
    }

    fn filter(&mut self, min_value: f64, max_value: f64) -> ShapeZooData {
        let rectangle = self.rectangle.filter(min_value,max_value);
        ShapeZooData {
            rectangle
        }
    }
}

#[derive(Clone,Debug)]
pub struct ShapeZoo {
    data: Arc<Mutex<ShapeZooData>>
}

impl ShapeZoo {
    pub fn new() -> ShapeZoo {
        ShapeZoo {
            data: Arc::new(Mutex::new(ShapeZooData::new()))
        }
    }

    pub fn rectangle(&self) -> MutexGuardRefMut<ShapeZooData,RectangleShapeSet> {
        MutexGuardRefMut::new(self.data.lock().unwrap()).map_mut(|x| &mut x.rectangle)
    }

    /*
    pub fn text(&mut self) -> MutexGuardRefMut<ShapeZooData,TextShapeSet> {
        MutexGuardRefMut::new(self.data.lock().unwrap()).map_mut(|x| &mut x.text)
    }
    */

    pub fn filter(&self, min_value: f64, max_value: f64) -> ShapeZoo {
        let data = lock!(self.data).filter(min_value,max_value);
        ShapeZoo {
            data: Arc::new(Mutex::new(data))
        }
    }
}
