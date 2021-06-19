use super::source::{ Source };
use super::program::{ ProgramBuilder };
use super::super::{ GLArity, GPUSpec, Precision, Phase };
use web_sys::{ WebGlUniformLocation, WebGlRenderingContext, WebGlProgram };
use keyed::keyed_handle;
use crate::webgl::util::handle_context_errors;
use crate::util::message::Message;

keyed_handle!(UniformHandle);

#[derive(Clone)]
pub(crate) struct UniformProto {
    precision: Precision,
    arity: GLArity,
    phase: Phase,
    name: String,
}

impl UniformProto {
    pub fn new_fragment(precision: Precision, arity: GLArity, name: &str) -> Box<UniformProto> {
        Box::new(UniformProto {
            precision, arity,
            name: name.to_string(),
            phase: Phase::Fragment
        })
    }

    pub fn new_vertex(precision: Precision, arity: GLArity, name: &str) -> Box<UniformProto> {
        Box::new(UniformProto {
            precision, arity,
            name: name.to_string(),
            phase: Phase::Vertex
        })
    }

    pub fn name(&self) -> &str { &self.name }
}

impl Source for UniformProto {
    fn cloned(&self) -> Box<dyn Source> { Box::new(self.clone()) }

    fn declare(&self, spec: &GPUSpec, phase: Phase) -> String {
        if phase != self.phase { return String::new(); }
        format!("uniform {} {};\n",spec.best_size(&self.precision,&self.phase).as_string(self.arity),self.name)
    }

    fn register(&self, builder: &mut ProgramBuilder) -> Result<(),Message> {
        builder.add_uniform(&self)
    }
}

#[derive(Clone)]
pub(crate) struct Uniform {
    proto: UniformProto,
    location: Option<WebGlUniformLocation>
}

impl Uniform {
    pub fn new(proto: &UniformProto, context: &WebGlRenderingContext, program: &WebGlProgram) -> Result<Uniform,Message> {
        let location = context.get_uniform_location(program,&proto.name);
        handle_context_errors(context)?;
        Ok(Uniform { proto: proto.clone(), location })
    }
}

pub(crate) struct UniformValues {
    valid: bool,
    gl_value: Vec<f32>,
    object: Uniform
}

impl UniformValues {
    pub(super) fn new(object: Uniform) -> UniformValues {
        UniformValues {
            valid: false,
            gl_value: vec![0.;object.proto.arity.to_num() as usize],
            object
        }
    }

    pub(super) fn activate(&self, context: &WebGlRenderingContext) -> Result<(),Message> {
        if !self.valid { return Ok(()); }
        if let Some(location) = &self.object.location {
            match self.object.proto.arity {
                GLArity::Scalar => context.uniform1f(Some(location),self.gl_value[0]),
                GLArity::Vec2 => context.uniform2f(Some(location),self.gl_value[0],self.gl_value[1]),
                GLArity::Vec3 => context.uniform3f(Some(location),self.gl_value[0],self.gl_value[1],self.gl_value[2]),
                GLArity::Vec4  => context.uniform4f(Some(location),self.gl_value[0],self.gl_value[1],self.gl_value[2],self.gl_value[3]),
                GLArity::Matrix4 => context.uniform_matrix4fv_with_f32_array(Some(location),false,&self.gl_value),
                GLArity::Sampler2D  => context.uniform1i(Some(location),self.gl_value[0] as i32)
            }
            handle_context_errors(context)?;
        }
        Ok(())
    }

    pub fn set_value(&mut self, our_value: &[f32]) -> Result<(),Message> {
        if self.gl_value.len() != our_value.len() {
            return Err(Message::CodeInvariantFailed(format!("uniform size mismatch {} type={} value={}",self.object.proto.name,self.object.proto.arity.to_num(),our_value.len())));
        }
        self.gl_value.copy_from_slice(&our_value);
        self.valid = true;
        Ok(())
    }

    pub fn discard(&mut self, _context: &WebGlRenderingContext) -> Result<(),Message> {
        Ok(())
    }
}
