use peregrine_core::Shape;

pub(crate) struct GLShape(Shape);

impl GLShape {
    pub fn new(shape: Shape) -> GLShape {
        GLShape(shape)
    }
}