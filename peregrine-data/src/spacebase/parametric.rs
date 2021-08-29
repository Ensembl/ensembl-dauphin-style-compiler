use std::{cmp::Ordering, sync::{Arc, Mutex}};

pub trait ParametricType {
    type Location;
    type Value;

    fn replace(&mut self, replace: &[(&Self::Location,Self::Value)]);
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct Variable(usize);

#[derive(Clone)]
pub enum ParameterValue<X: Clone> {
    Constant(X),
    Variable(Variable,X)
}

#[cfg(debug_assertions)]
impl<X: Clone+std::fmt::Debug> std::fmt::Debug for ParameterValue<X> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParameterValue::Constant(x) => write!(f,"Constant({:?})",x),
            ParameterValue::Variable(v,x) => write!(f,"Variable({:?},{:?})",v,x),
        }
    }
}

impl<X:Clone> ParameterValue<X> {
    pub fn param_default(&self) -> &X {
        match self {
            ParameterValue::Constant(x) => x,
            ParameterValue::Variable(_,x) => x
        }
    }
}

impl<X: PartialEq + Clone> PartialEq for ParameterValue<X> {
    fn eq(&self, other: &ParameterValue<X>) -> bool {
        self.param_default() == other.param_default()
    }
}

impl<X: PartialOrd + Clone> PartialOrd for ParameterValue<X> {
    fn partial_cmp(&self, other: &ParameterValue<X>) -> Option<Ordering> {
        self.param_default().partial_cmp(other.param_default())
    }
}

pub trait Flattenable {
    type Location;
    type Target;

    fn extract(&self) -> (Self::Target,Substitutions<Self::Location>) {
        let mut subs = Substitutions::empty();
        let out = self.flatten(&mut subs,|x| x);
        (out,subs)
    }

    fn flatten<F,L>(&self, subs: &mut Substitutions<L>, cb: F) -> Self::Target where F: Fn(Self::Location) -> L;
}

#[derive(Clone)]
pub struct VariableValues<X> {
    values: Arc<Mutex<Vec<X>>>
}

impl<X: Clone> VariableValues<X> {
    pub fn new() -> VariableValues<X> {
        VariableValues {
            values: Arc::new(Mutex::new(vec![]))
        }
    }

    pub fn new_variable(&self, value: X) -> Variable {
        let mut vars = self.values.lock().unwrap();
        let out = vars.len();
        vars.push(value);
        Variable(out)
    }

    pub fn update_variable(&self, var: &Variable, value: X) {
        self.values.lock().unwrap()[var.0] = value;
    }

    fn get_values(&self, vars: &[&Variable]) -> Vec<Option<X>> {
        vars.iter().map(|x| {
            self.values.lock().unwrap().get(x.0).cloned()
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

    pub(super) fn flatten<X: Clone, F>(&mut self, data: &[ParameterValue<X>], cb: F) -> Vec<X> where F: Fn(usize) -> L {
        let mut out = vec![];
        for (i,item) in data.iter().enumerate() {
            match item {
                ParameterValue::Constant(v) => { 
                    out.push(v.clone());
                }
                ParameterValue::Variable(var,initial) => {
                    self.locations.push((cb(i),var.clone()));
                    out.push(initial.clone());
                }
            }
        }
        out
    }

    pub fn apply<X: Clone>(&self, target: &mut dyn ParametricType<Location=L,Value=X>, values: &VariableValues<X>) {
        let vars = self.locations.iter().map(|x| &x.1).collect::<Vec<_>>();
        let mut values = values.get_values(&vars);
        let mut subs = vec![];
        for ((location,_),value) in self.locations.iter().zip(values.drain(..)) {
            if let Some(value) = value {
                subs.push((location,value));
            }
        }
        target.replace(&subs);
    }
}
