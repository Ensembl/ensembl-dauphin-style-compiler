/* The parametric types are designed to help with vector data with variable values (among constant ones). For example,
 * (schematically) [1,2,3,X,7,8,9]. When X=0 this equals [1,2,3,0,7,8,9], when X=5 this equals [1,2,3,5,7,8,9].
 * They are designed to be used for WebGL co-ordinates for objects which "move", such as spectres. 
 *
 * Types supporting parameterisation nominate a type to act as its "location" be that a simple usize index or an enum.
 * They then implement ParametricType<Location>; ParametricType::Value=X where X is the type of values to store in such
 * locations. There is asingle method replace() which should apply that substituion.
 * 
 * When building a template, the structure should have type ParamaterValue<Value>. This supplies arms Constant and
 * Variable to allow specifying the tamplate. The variable arm has an initial value and a Variable which identifies the
 * variable. Such Variables can only be allocated from a VariableValues struct which as well as allocating new 
 * Variables, holds the variable values for substitutions.
 * 
 * The parameterisable type should implement Flattenable. This allows the conversion between a parameterised and a
 * non-parameterised type with the (implemented) extract() method. It returns the corresponding non-parameterised data
 * structure, useing initial values for variable positions, and also an instance of Substitutions<Location>. The
 * Substitutions method contains all the positions to be substituted and whenever passed the target and an instance of
 * VariableValues will substitute the target.
 *
 * To implement Flattenable, the parameterised type needs to implement the flatten() method. This is called once per
 * extract but is passed two arguments, to allow recursive flattening (parameterisable data-structrues inside other
 * parameterisable data-structures).
 */

use std::{cmp::Ordering, sync::{Arc, Mutex}};

use crate::EachOrEvery;

pub trait ParametricType<Location> {
    type Value;

    fn replace(&mut self, replace: &[(&Location,Self::Value)]);
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

pub trait Flattenable<Location> {
    type Target;

    fn extract(&self) -> (Self::Target,Substitutions<Location>) {
        let mut subs = Substitutions::empty();
        let out = self.flatten(&mut subs,|x| x);
        (out,subs)
    }

    fn flatten<F,L>(&self, subs: &mut Substitutions<L>, cb: F) -> Self::Target where F: Fn(Location) -> L;
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

#[cfg(debug_assertions)]
impl<L> std::fmt::Debug for Substitutions<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Substitutions(...)")
    }
}

impl<L> Substitutions<L> {
    pub(super) fn empty() -> Substitutions<L> {
        Substitutions {
            locations: vec![]
        }
    }

    pub fn len(&self) -> usize { self.locations.len() }

    pub fn apply<X: Clone>(&self, target: &mut dyn ParametricType<L,Value=X>, values: &VariableValues<X>) {
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

    pub fn add_location<'a,X: Clone>(&mut self, location: L, item: &'a ParameterValue<X>) -> &'a X {
        match item {
            ParameterValue::Constant(v) => {
                v
            }
            ParameterValue::Variable(var,initial) => {
                self.locations.push((location,var.clone()));
                initial
            }
        }
    }
}

impl<X: Clone> Flattenable<usize> for [ParameterValue<X>] {
    type Target = Vec<X>;

    fn flatten<F,L>(&self, subs: &mut Substitutions<L>, cb: F) -> Self::Target where F: Fn(usize) -> L {
        let mut out = vec![];
        for (i,item) in self.iter().enumerate() {
            out.push(subs.add_location(cb(i),item).clone());
        }
        out
    }
}

impl<X: Clone> Flattenable<usize> for EachOrEvery<ParameterValue<X>> {
    type Target = EachOrEvery<X>;

    fn flatten<F,L>(&self, subs: &mut Substitutions<L>, cb: F) -> Self::Target where F: Fn(usize) -> L {
        self.enumerated_map(|i,item| {
            subs.add_location(cb(i),item).clone()
        })
    }
}
