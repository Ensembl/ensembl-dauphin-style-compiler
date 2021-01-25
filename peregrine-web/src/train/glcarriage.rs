use peregrine_core::{ Carriage, CarriageId };
use crate::shape::core::glshape::GLShape;
use std::hash::{ Hash, Hasher };
use std::sync::Mutex;

pub(crate) struct GLCarriage {
    id: CarriageId,
    opacity: Mutex<f64>,
    shapes: Vec<GLShape>
}

impl PartialEq for GLCarriage {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for GLCarriage {}

impl Hash for GLCarriage {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.id.hash(hasher);
    }
}

impl GLCarriage {
    pub fn new(carriage: &Carriage, opacity: f64) -> GLCarriage {
        GLCarriage {
            shapes: carriage.shapes().iter().map(|x| GLShape::new(x.clone())).collect(),
            id: carriage.id().clone(),
            opacity: Mutex::new(opacity)
        }
    }

    pub fn id(&self) -> &CarriageId { &self.id }

    pub(super) fn set_opacity(&self, amount: f64) {
        *self.opacity.lock().unwrap() = amount;
    }

    pub fn destroy(&self) {

    }
}