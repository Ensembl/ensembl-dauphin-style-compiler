use anyhow::{ bail };
use super::source::{ Source };
use super::program::Program;
use super::super::{ GLArity, GPUSpec, Precision, Phase };
use super::values::ProcessValueType;
use web_sys::{ WebGlUniformLocation, WebGlRenderingContext, WebGlBuffer };
use crate::process_value_handle;

process_value_handle!(UniformHandle);

#[derive(Clone)]
pub(crate) struct Uniform {
    precision: Precision,
    arity: GLArity,
    phase: Phase,
    name: String
}

impl Uniform {
    pub fn new_fragment(precision: Precision, arity: GLArity, name: &str) -> Box<Uniform> {
        Box::new(Uniform {
            precision, arity,
            name: name.to_string(),
            phase: Phase::Fragment
        })
    }

    pub fn new_vertex(precision: Precision, arity: GLArity, name: &str) -> Box<Uniform> {
        Box::new(Uniform {
            precision, arity,
            name: name.to_string(),
            phase: Phase::Vertex
        })
    }
}

impl Source for Uniform {
    fn declare(&self, spec: &GPUSpec, phase: Phase) -> String {
        if phase != self.phase { return String::new(); }
        format!("uniform {} {};\n",spec.best_size(&self.precision,&self.phase).as_string(self.arity),self.name)
    }

    fn build(&self, program: &mut Program) -> anyhow::Result<()> {
        program.add_uniform(&self)
    }
}

impl ProcessValueType for Uniform {
    type GLKey = WebGlUniformLocation;
    type GLValue = Vec<f32>;
    type OurValue = Vec<f32>;

    fn name(&self) -> &str { &self.name }

    fn activate(&self, context: &WebGlRenderingContext, gl_key: &WebGlUniformLocation, gl_value: &Vec<f32>) -> anyhow::Result<()> {
        match gl_value.len() {
            1 => context.uniform1f(Some(gl_key),gl_value[0]),
            2 => context.uniform2f(Some(gl_key),gl_value[0],gl_value[1]),
            3 => context.uniform3f(Some(gl_key),gl_value[0],gl_value[1],gl_value[2]),
            4 => context.uniform4f(Some(gl_key),gl_value[0],gl_value[1],gl_value[2],gl_value[3]),
            x => bail!("bad uniform size {}",x)
        }
        Ok(())
    }

    fn value_to_gl(&self, _context: &WebGlRenderingContext, our_data: Self::OurValue) -> anyhow::Result<Self::GLValue> {
        Ok(our_data)
    }

    fn delete(&self, context: &WebGlRenderingContext, gl_value: &Self::GLValue) -> anyhow::Result<()> {
        Ok(())
    }
}
