use super::source::Source;
use super::super::{ GLArity, GPUSpec, Precision, Phase };

pub(crate) struct Varying {
    precision: Precision,
    arity: GLArity,
    name: String
}

impl Varying {
    pub fn new(precision: Precision, arity: GLArity, name: &str) -> Box<Varying> {
        Box::new(Varying {
            precision, arity,
            name: name.to_string(),
        })
    }
}

impl Source for Varying {
    fn declare(&self, spec: &GPUSpec, _phase: Phase) -> String {
        format!("varying {} {};\n",spec.best_size(&self.precision,&Phase::Vertex).as_string(self.arity),self.name)
    }  
}