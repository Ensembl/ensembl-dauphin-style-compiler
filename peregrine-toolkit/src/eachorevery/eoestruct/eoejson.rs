use std::collections::{HashMap, HashSet};
use crate::eachorevery::EachOrEvery;
use super::{eoestruct::{StructConst, StructError, struct_error, StructVarGroup, LateValues}, structtemplate::{StructVar, StructPair}, StructTemplate, eoestructdata::{DataStackTransformer, eoestack_run}, structbuilt::StructBuilt};
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

pub fn struct_to_json(input: &StructBuilt, lates: Option<&LateValues>) -> Result<JsonValue,StructError> {
    eoestack_run(input,lates,JsonTransformer)
}

fn to_var_type<F,X>(input: &[JsonValue], cb: F) -> Result<EachOrEvery<X>,StructError> where F: Fn(&JsonValue) -> Option<X> {
    let values = input.iter().map(cb).collect::<Option<Vec<_>>>();
    values.map(|x| EachOrEvery::each(x)).ok_or(struct_error("non-homogenous variable"))
}

pub(super) fn array_to_var(group: &mut StructVarGroup, values: &[JsonValue]) -> Result<StructVar,StructError> {
    Ok(if let Some(first) = values.first() {
        match first {
            JsonValue::Bool(_) => {
                StructVar::new_boolean(group,to_var_type(values,|x| {
                    if let JsonValue::Bool(x) = x { Some(*x) } else { None }
                })?)
            },
            JsonValue::Number(_) => {
                StructVar::new_number(group,to_var_type(values,|x| {
                    if let JsonValue::Number(x) = x { Some(x.as_f64().unwrap()) } else { None }
                })?)
            },
            JsonValue::String(_) => {
                StructVar::new_string(group,to_var_type(values,|x| {
                    if let JsonValue::String(x) = x { Some(x.to_string()) } else { None }
                })?)
            },
            _ => StructVar::new_boolean(group,EachOrEvery::each(vec![])) // XXX error
        }
    } else {
        StructVar::new_boolean(group,EachOrEvery::each(vec![])) // XXX error
    })

}

struct EoeFromJson {
    specs: HashSet<String>,
    ifs: HashSet<String>,
    vars: Vec<HashMap<String,StructVar>>,
    lates: Vec<(String,StructVar)>
}

impl EoeFromJson {
    fn new(mut specs: Vec<String>, mut ifs: Vec<String>, json: &JsonValue) ->  Result<(StructTemplate,Vec<(String,StructVar)>),StructError> {
        let mut obj = EoeFromJson{
            specs: specs.drain(..).collect(),
            ifs: ifs.drain(..).collect(),
            vars: vec![],
            lates: vec![]
        };
        let template = obj.build(json)?;
        Ok((template,obj.lates))
    }

    fn to_var(&mut self, group: &mut StructVarGroup, key: &str, input: &JsonValue) -> Result<StructVar,StructError> {
        let values = match input {
            JsonValue::Array(x) => x.as_slice(),
            JsonValue::Null => {
                let late = StructVar::new_late(group);
                self.lates.push((key.to_string(), late.clone()));
                return Ok(late);
            },
            _ => &[]
        };
        array_to_var(group,values)
    }
    
    fn to_all(&mut self, map: &Map<String,JsonValue>) -> Result<Option<StructTemplate>,StructError> {
        let mut group = StructVarGroup::new();
        let mut expr = None;
        for key in map.keys() {
            if self.specs.contains(key) { expr = Some(key); break; }
        }
        let expr = if let Some(expr) = expr { expr } else { return Ok(None); };
        let mut vars = vec![];
        let mut var_names = HashMap::new();
        for (key,value) in map.iter() {
            if key == expr { continue; }
            let var = self.to_var(&mut group,key,&value)?;
            vars.push(var.clone());
            var_names.insert(key.clone(),var);
        }
        self.vars.push(var_names);
        let expr = self.build(map.get(expr).unwrap())?; // expr guranteed in map during setting
        self.vars.pop();
        Ok(Some(StructTemplate::new_all(&mut group,expr)))
    }

    fn to_condition(&mut self, map: &Map<String,JsonValue>) -> Result<Option<StructTemplate>,StructError> {
        let mut expr = None;
        for key in map.keys() {
            if self.ifs.contains(key) { expr = Some(key); break; }
        }
        let expr = if let Some(expr) = expr { expr } else { return Ok(None); };
        let value = self.build(map.get(expr).unwrap())?; // expr guranteed in map during setting
        for map in self.vars.iter().rev() {
            if let Some(var) = map.get(expr) {
                return Ok(Some(StructTemplate::new_condition(var.clone(),value)));
            }
        }
        Ok(None)
    }

    fn build(&mut self, json: &JsonValue) ->  Result<StructTemplate,StructError> {
        Ok(match json {
            JsonValue::Null => StructTemplate::new_null(),
            JsonValue::Bool(x) => StructTemplate::new_boolean(x.clone()),
            JsonValue::Number(x) => StructTemplate::new_number(x.as_f64().unwrap()),
            JsonValue::String(x) => {
                for map in self.vars.iter().rev() {
                    if let Some(var) = map.get(x) {
                        return Ok(StructTemplate::new_var(var));
                    }
                }
                StructTemplate::new_string(x.clone())
            },
            JsonValue::Array(x) => {
                let values = x.iter().map(|x| self.build(x)).collect::<Result<_,_>>()?;
                StructTemplate::new_array(EachOrEvery::each(values))
            },
            JsonValue::Object(x) => {
                if let Some(all) = self.to_all(&x)? {
                    all
                } else if let Some(cond) = self.to_condition(&x)? {
                    cond
                } else {
                    StructTemplate::new_object(EachOrEvery::each(x.iter().map(|(k,v)|{
                        Ok::<StructPair,StructError>(StructPair(k.to_string(),self.build(v)?))
                    }).collect::<Result<_,_>>()?))
                }
            }
        })
    }
}

pub fn struct_from_json(alls: Vec<String>, ifs: Vec<String>, json: &JsonValue) -> Result<(StructTemplate,Vec<(String,StructVar)>),StructError> {
    EoeFromJson::new(alls,ifs,json)
}
