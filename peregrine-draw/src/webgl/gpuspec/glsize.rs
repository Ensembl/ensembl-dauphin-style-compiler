use super::glarity::GLArity;
use web_sys::WebGlRenderingContext;

#[derive(Clone,Copy)]
pub(crate) enum GLSize {
    FloatHigh, FloatMed, FloatLow,
    IntHigh,   IntMed,   IntLow
}

impl GLSize {
    pub(super) fn is_int(&self) -> bool {
        match self {
            GLSize::IntHigh|GLSize::IntMed|GLSize::IntLow => true,
            _ => false
        }
    }

    pub(crate) fn as_string(&self, arity: GLArity) -> String {
        let prec_str = match self {
            GLSize::FloatHigh|GLSize::IntHigh => "highp",
            GLSize::FloatMed|GLSize::IntMed => "mediump",
            GLSize::FloatLow|GLSize::IntLow => "lowp",
        };
        let type_str = if self.is_int() {
            match arity {
                GLArity::Scalar => "int",
                GLArity::Vec2 => "ivec2",
                GLArity::Vec3 => "ivec3",
                GLArity::Vec4 => "ivec4",
                GLArity::Matrix4 => "",
                GLArity::Sampler2D => ""
            }
        } else {
            match arity {
                GLArity::Scalar => "float",
                GLArity::Vec2 => "vec2",
                GLArity::Vec3 => "vec3",
                GLArity::Vec4 => "vec4",
                GLArity::Matrix4 => "mat4",
                GLArity::Sampler2D => "sampler2D"
            }

        };
        format!("{} {}",prec_str,type_str)
    }

    pub fn get_gltype(&self) -> u32 {
        match self {
            GLSize::FloatHigh => WebGlRenderingContext::HIGH_FLOAT,
            GLSize::FloatMed => WebGlRenderingContext::MEDIUM_FLOAT,
            GLSize::FloatLow => WebGlRenderingContext::LOW_FLOAT,
            GLSize::IntHigh => WebGlRenderingContext::HIGH_INT,
            GLSize::IntMed => WebGlRenderingContext::MEDIUM_INT,
            GLSize::IntLow => WebGlRenderingContext::LOW_INT,
        }
    }
}