#[derive(Clone,Copy,PartialEq,Eq)]
pub enum GLArity {
    Scalar,
    Vec2,
    Vec3,
    Vec4,
    Sampler2D
}

impl GLArity {
    pub fn to_num(&self) -> u8 {
        match self {
            GLArity::Scalar => 1,
            GLArity::Vec2 => 2,
            GLArity::Vec3 => 3,
            GLArity::Vec4 => 4,
            GLArity::Sampler2D => 1
        }
    }
}