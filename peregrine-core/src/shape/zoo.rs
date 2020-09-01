use std::sync::{ Arc, Mutex };
use super::rectangle::RectangleShapeSet;
use owning_ref::MutexGuardRefMut;

pub struct ShapeZooData {
    rectangle: RectangleShapeSet
}

pub struct ShapeZoo {
    data: Arc<Mutex<ShapeZooData>>
}

impl ShapeZoo {
    pub fn rectangle(&mut self) -> MutexGuardRefMut<ShapeZooData,RectangleShapeSet> {
        MutexGuardRefMut::new(self.data.lock().unwrap()).map_mut(|x| &mut x.rectangle)
    }

    /*
    pub fn text(&mut self) -> MutexGuardRefMut<ShapeZooData,TextShapeSet> {
        MutexGuardRefMut::new(self.data.lock().unwrap()).map_mut(|x| &mut x.text)
    }
    */
}
