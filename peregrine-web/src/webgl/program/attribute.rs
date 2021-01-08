use super::source::{ Source, Runtime };
use super::super::{ GLArity, GPUSpec, Precision, Phase };

pub(crate) struct RuntimeAttribute {

}

impl Runtime for RuntimeAttribute {

}

pub(crate) struct Attribute {
    precision: Precision,
    arity: GLArity,
    name: String
}

impl Attribute {
    pub fn new(precision: Precision, arity: GLArity, name: &str) -> Box<Attribute> {
        Box::new(Attribute {
            precision, arity,
            name: name.to_string(),
        })
    }
}

impl Source for Attribute {
    fn declare(&self, spec: &GPUSpec, phase: Phase) -> String {
        if phase != Phase::Vertex { return String::new(); }
        format!("attribute {} {};\n",spec.best_size(&self.precision,&Phase::Vertex).as_string(self.arity),self.name)
    }

    fn to_binary(&self) -> Box<dyn Runtime> {
        Box::new(RuntimeAttribute {})
    }
}