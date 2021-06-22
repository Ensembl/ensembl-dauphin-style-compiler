use std::sync::{Arc, Mutex};

pub trait ParametricType {
    type Location;
    type Value;

    fn replace(&mut self, replace: &[(&Self::Location,Self::Value)]);
}

#[derive(Clone)]
pub struct Variable(usize);

pub enum ParameterValue<X> {
    Constant(X),
    Variable(Variable)
}

#[derive(Clone)]
pub struct VariableValues<X> {
    values: Arc<Mutex<Vec<X>>>
}

impl<X: Clone> VariableValues<X> {
    fn new() -> VariableValues<X> {
        VariableValues {
            values: Arc::new(Mutex::new(vec![]))
        }
    }

    fn new_variable(&self, value: X) -> Variable {
        let mut vars = self.values.lock().unwrap();
        let out = vars.len();
        vars.push(value);
        Variable(out)
    }

    fn update_variable(&self, var: &Variable, value: X) {
        self.values.lock().unwrap()[var.0] = value;
    }

    fn get_values(&self, vars: &[&Variable]) -> Vec<X> {
        vars.iter().map(|x| {
            self.values.lock().unwrap()[x.0].clone()
        }).collect()
    }
}

pub struct Substitutions<L> {
    locations: Vec<(L,Variable)>
}

impl<L> Substitutions<L> {
    pub(super) fn empty() -> Substitutions<L> {
        Substitutions {
            locations: vec![]
        }
    }

    pub(super) fn flatten<X: Clone+Default, F>(&mut self, data: &[ParameterValue<X>], cb: F) -> Vec<X> where F: Fn(usize) -> L {
        let mut out = vec![];
        for (i,item) in data.iter().enumerate() {
            match item {
                ParameterValue::Constant(v) => { 
                    out.push(v.clone());
                }
                ParameterValue::Variable(var) => {
                    self.locations.push((cb(i),var.clone()));
                    out.push(Default::default());
                }
            }
        }
        out
    }

    fn apply<X: Clone>(&self, target: &mut dyn ParametricType<Location=L,Value=X>, values: VariableValues<X>) {
        let vars = self.locations.iter().map(|x| &x.1).collect::<Vec<_>>();
        let mut values = values.get_values(&vars);
        let mut subs = vec![];
        for ((location,_),value) in self.locations.iter().zip(values.drain(..)) {
            subs.push((location,value));
        }
        target.replace(&subs);
    }
}
