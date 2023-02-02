use std::{sync::Arc, collections::BTreeMap};
use eachorevery::eoestruct::{StructConst, StructValue};

fn const_matches(a: &StructValue, b: &StructConst) -> bool {
    if let StructValue::Const(a) = a {
        a == b
    } else {
        false
    }
}

fn template_iter(data: &StructValue) ->Vec<StructValue> {
    match data {
        StructValue::Array(a) => {
            a.as_ref().clone()
        },
        _ => { vec![] }
    }
}

fn template_dict(data: &StructValue) -> BTreeMap<String,StructValue> {
    match data {
        StructValue::Object(a) => {
            a.as_ref().clone()
        },
        _ => { BTreeMap::new() }
    }
}

fn member(old: &StructValue, value: &StructConst, yn: bool) -> StructValue {
    let mut out = vec![];
    if yn {
        /* insert */
        let duplicate = template_iter(old).drain(..).any(|x| const_matches(&x,value));
        if !duplicate {
            out.push(StructValue::Const(value.clone()));
        }
        out.sort();
    } else {
        /* remove */
        out = template_iter(old).drain(..)
            .filter(|x| !const_matches(x,value))
            .collect();
    }
    StructValue::Array(Arc::new(out))
}

fn set(old: &StructValue, key: &str, value: &StructValue) -> StructValue {
    let mut dict = template_dict(old);
    dict.insert(key.to_string(),value.clone());
    StructValue::Object(Arc::new(dict))
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub enum SettingMode {
    KeyValue(String,StructValue),
    Set(bool),
    Member(String,bool),
    None
}

impl SettingMode {
    pub fn update(&self, old: StructValue) -> StructValue {
        let out = match self {
            SettingMode::KeyValue(key,value) => {
                set(&old,key,value)
            },
            SettingMode::Set(value) => {
                StructValue::new_boolean(*value)
            },
            SettingMode::Member(value, yn) => {
                member(&old,&StructConst::String(value.to_string()),*yn)
            },
            SettingMode::None => { old.clone() }
        };
        out
    }
}
