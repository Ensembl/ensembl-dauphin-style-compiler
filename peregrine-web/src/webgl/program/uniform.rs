use anyhow::{ bail, anyhow as err };
use super::source::{ Source };
use super::program::Program;
use super::super::{ GLArity, GPUSpec, Precision, Phase };
use web_sys::{ WebGlUniformLocation, WebGlRenderingContext };
use crate::keyed_handle;
use crate::webgl::util::handle_context_errors;

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

    fn build(&mut self, program: &mut Program) -> anyhow::Result<()> {
        let context = program.context();
        let location = context.get_uniform_location(program.program(),self.name());
        handle_context_errors(context)?;
        self.location = Some(location.ok_or_else(|| err!("cannot get uniform '{}'",self.name))?);
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

    pub(super) fn activate(&self, context: &WebGlRenderingContext) -> anyhow::Result<()> {
        if let Some(gl_value) = &self.gl_value {
            let gl_value : Vec<_> = gl_value.iter().map(|x| *x as f32).collect();
            let location = self.object.location.as_ref().unwrap();
            match gl_value.len() {
                1 => context.uniform1f(Some(location),gl_value[0]),
                2 => context.uniform2f(Some(location),gl_value[0],gl_value[1]),
                3 => context.uniform3f(Some(location),gl_value[0],gl_value[1],gl_value[2]),
                4 => context.uniform4f(Some(location),gl_value[0],gl_value[1],gl_value[2],gl_value[3]),
                x => bail!("bad uniform size {}",x)
            }
            handle_context_errors(context)?;
        }
        Ok(())
    }

    pub fn set_value(&mut self, context: &WebGlRenderingContext, our_value: Vec<f64>) -> anyhow::Result<()> {
        self.gl_value = Some(our_value);
        Ok(())
    }
}
