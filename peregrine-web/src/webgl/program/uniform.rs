use super::source::{ Source, Runtime };
use super::super::{ GLArity, GPUSpec, Precision, Phase };

pub(crate) struct RuntimeUniform {

}

impl Runtime for RuntimeUniform {
    
}

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
    
    fn to_binary(&self) -> Box<dyn Runtime> {
        Box::new(RuntimeUniform {})
    }
}