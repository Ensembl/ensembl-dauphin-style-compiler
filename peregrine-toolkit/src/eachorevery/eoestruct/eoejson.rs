use std::collections::{HashMap, HashSet};

use crate::eachorevery::EachOrEvery;

use super::{eoestruct::{StructConst, Struct, StructPair, StructError, struct_error}, templatetree::StructVar, StructTemplate, expand::StructBuilt, eoestructdata::{DataStackTransformer, eoestack_run}};
use serde_json::{Value as JsonValue, Number, Map};
struct JsonTransformer;

impl DataStackTransformer<StructConst,JsonValue> for JsonTransformer {
    fn make_singleton(&mut self, value: StructConst) -> JsonValue {
        match value {
            StructConst::Number(input) => JsonValue::Number(Number::from_f64(input).unwrap()),
            StructConst::String(input) => JsonValue::String(input),
            StructConst::Boolean(input) => JsonValue::Bool(input),
            StructConst::Null => JsonValue::Null
        }
    }

    fn make_array(&mut self, value: Vec<JsonValue>) -> JsonValue {
        JsonValue::Array(value)
    }

    fn make_object(&mut self, value: Vec<(String,JsonValue)>) -> JsonValue {
        JsonValue::Object(value.iter().map(|x| x.clone()).collect())
    }
}

pub fn struct_to_json(input: &StructBuilt) -> Result<JsonValue,StructError> {
    eoestack_run(input,JsonTransformer)
}

fn to_var_type<F,X>(input: &[JsonValue], cb: F) -> Result<EachOrEvery<X>,StructError> where F: Fn(&JsonValue) -> Option<X> {
    let values = input.iter().map(cb).collect::<Option<Vec<_>>>();
    values.map(|x| EachOrEvery::each(x)).ok_or(struct_error("non-homogenous variable"))
}
// XXX conditionals
// XXX any iter
fn to_var(input: &JsonValue) -> Result<StructVar,StructError> {
    let values = match input {
        JsonValue::Array(x) => x.as_slice(),
        _ => &[]
    };
    Ok(if let Some(first) = values.first() {
        match first {
            JsonValue::Bool(_) => {
                StructVar::new_boolean(to_var_type(values,|x| {
                    if let JsonValue::Bool(x) = x { Some(*x) } else { None }
                })?)
            },
            JsonValue::Number(_) => {
                StructVar::new_number(to_var_type(values,|x| {
                    if let JsonValue::Number(x) = x { Some(x.as_f64().unwrap()) } else { None }
                })?)
            },
            JsonValue::String(_) => {
                StructVar::new_string(to_var_type(values,|x| {
                    if let JsonValue::String(x) = x { Some(x.to_string()) } else { None }
                })?)
            },
            _ => StructVar::new_boolean(EachOrEvery::each(vec![]))
        }
    } else {
        StructVar::new_boolean(EachOrEvery::each(vec![]))
    })
}

struct EoeFromJson {
    specs: HashSet<String>,
    vars: Vec<HashMap<String,StructVar>>
}

impl EoeFromJson {
    fn new(mut specs: Vec<String>, json: &JsonValue) ->  Result<StructTemplate,StructError> {
        let mut obj = EoeFromJson{
            specs: specs.drain(..).collect(),
            vars: vec![]
        };
        obj.build(json)
    }

    fn to_all(&mut self, map: &Map<String,JsonValue>) -> Result<Option<StructTemplate>,StructError> {
        let mut expr = None;
        for key in map.keys() {
            if self.specs.contains(key) { expr = Some(key); break; }
        }
        let expr = if let Some(expr) = expr { expr } else { return Ok(None); };
        let mut vars = vec![];
        let mut var_names = HashMap::new();
        for (key,value) in map.iter() {
            if key == expr { continue; }
            let var = to_var(&value)?;
            vars.push(var.clone());
            var_names.insert(key.clone(),var);
        }
        self.vars.push(var_names);
        let expr = self.build(map.get(expr).unwrap())?; // expr guranteed in map during setting
        self.vars.pop();
        Ok(Some(Struct::new_all(&vars,expr)))
    }

    fn build(&mut self, json: &JsonValue) ->  Result<StructTemplate,StructError> {
        Ok(match json {
            JsonValue::Null => Struct::new_null(),
            JsonValue::Bool(x) => Struct::new_boolean(x.clone()),
            JsonValue::Number(x) => Struct::new_number(x.as_f64().unwrap()),
            JsonValue::String(x) => {
                for map in self.vars.iter().rev() {
                    if let Some(var) = map.get(x) {
                        return Ok(Struct::new_var(var.clone()));
                    }
                }
                Struct::new_string(x.clone())
            },
            JsonValue::Array(x) => {
                let values = x.iter().map(|x| self.build(x)).collect::<Result<_,_>>()?;
                Struct::new_array(values)
            },
            JsonValue::Object(x) => {
                if let Some(all) = self.to_all(&x)? {
                    all
                } else {
                    Struct::new_object(x.iter().map(|(k,v)|{
                        Ok::<StructPair<_>,StructError>(StructPair(k.to_string(),self.build(v)?))
                    }).collect::<Result<_,_>>()?)
                }
            }
        })
    }
}

pub fn struct_from_json(specs: Vec<String>, json: &JsonValue) -> Result<StructTemplate,StructError> {
    EoeFromJson::new(specs,json)
}
