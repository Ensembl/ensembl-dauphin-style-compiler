use anyhow::{ bail };
use super::source::{ Source };
use super::program::Program;
use super::super::{ GLArity, GPUSpec, Precision, Phase };
use web_sys::{ WebGlUniformLocation, WebGlRenderingContext };
use keyed::keyed_handle;
use crate::webgl::util::handle_context_errors;
use crate::util::message::Message;

keyed_handle!(UniformHandle);

#[derive(Clone)]
pub(crate) struct Uniform {
    precision: Precision,
    arity: GLArity,
    phase: Phase,
    name: String,
    location: Option<WebGlUniformLocation>
}

impl Uniform {
    pub fn new_fragment(precision: Precision, arity: GLArity, name: &str) -> Box<Uniform> {
        Box::new(Uniform {
            precision, arity,
            name: name.to_string(),
            phase: Phase::Fragment,
            location: None
        })
    }

    pub fn new_vertex(precision: Precision, arity: GLArity, name: &str) -> Box<Uniform> {
        Box::new(Uniform {
            precision, arity,
            name: name.to_string(),
            phase: Phase::Vertex,
            location: None
        })
    }

    pub fn name(&self) -> &str { &self.name }
}

impl Source for Uniform {
    fn cloned(&self) -> Box<dyn Source> { Box::new(self.clone()) }

    fn declare(&self, spec: &GPUSpec, phase: Phase) -> String {
        if phase != self.phase { return String::new(); }
        format!("uniform {} {};\n",spec.best_size(&self.precision,&self.phase).as_string(self.arity),self.name)
    }

    fn build(&mut self, context: &WebGlRenderingContext, program: &mut Program) -> Result<(),Message> {
        self.location = context.get_uniform_location(program.program(),self.name());
        handle_context_errors(context)?;
        program.add_uniform(&self)
    }
}

pub(crate) struct UniformValues {
    gl_value: Option<Vec<f64>>,
    object: Uniform
}

impl UniformValues {
    pub(super) fn new(object: Uniform) -> UniformValues {
        UniformValues {
            gl_value: None,
            object
        }
    }

    pub(super) fn activate(&self, context: &WebGlRenderingContext) -> Result<(),Message> {
        if let Some(gl_value) = &self.gl_value {
            let gl_value : Vec<_> = gl_value.iter().map(|x| *x as f32).collect();
            if gl_value.len() != self.object.arity.to_num() as usize {
                return Err(Message::CodeInvariantFailed(format!("uniform size mismatch {} type={} value={}",self.object.name,self.object.arity.to_num(),gl_value.len())));
            }
            if let Some(location) = &self.object.location {
                match self.object.arity {
                    GLArity::Scalar => context.uniform1f(Some(location),gl_value[0]),
                    GLArity::Vec2 => context.uniform2f(Some(location),gl_value[0],gl_value[1]),
                    GLArity::Vec3 => context.uniform3f(Some(location),gl_value[0],gl_value[1],gl_value[2]),
                    GLArity::Vec4  => context.uniform4f(Some(location),gl_value[0],gl_value[1],gl_value[2],gl_value[3]),
                    GLArity::Matrix4 => context.uniform_matrix4fv_with_f32_array(Some(location),false,&gl_value),
                    GLArity::Sampler2D  => context.uniform1i(Some(location),gl_value[0] as i32)
                }
                handle_context_errors(context)?;
            }
        }
        Ok(())
    }

    pub fn set_value(&mut self, our_value: Vec<f64>) -> Result<(),Message> {
        self.gl_value = Some(our_value);
        Ok(())
    }

    pub fn discard(&mut self, _context: &WebGlRenderingContext) -> Result<(),Message> {
        Ok(())
    }
}
