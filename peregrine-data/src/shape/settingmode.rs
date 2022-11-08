use std::sync::Arc;
use peregrine_toolkit::{eachorevery::{eoestruct::{StructBuilt, StructTemplate, StructConst}}, error::Error};

fn const_matches(a: &StructTemplate, b: &StructConst) -> bool {
    if let StructTemplate::Const(a) = a {
        a == b
    } else {
        false
    }
}

fn template_iter(data: &StructTemplate) ->Vec<StructTemplate> {
    match data {
        StructTemplate::Array(a) => {
            a.as_ref().clone()
        },
        _ => { vec![] }
    }
}

fn member(old: &StructTemplate, value: &StructConst, yn: bool) -> Result<StructTemplate,Error> {
    let mut out = vec![];
    if yn {
        /* insert */
        let duplicate = template_iter(old).drain(..).any(|x| const_matches(&x,value));
        if !duplicate {
            out.push(StructTemplate::Const(value.clone()));
        }
    } else {
        /* remove */
        out = template_iter(old).drain(..)
            .filter(|x| !const_matches(x,value))
            .collect();
    }
    Ok(StructTemplate::Array(Arc::new(out)))
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub enum SettingMode {
    Set(bool),
    Member(String,bool),
    None
}

impl SettingMode {
    pub fn update(&self, old: StructBuilt) -> Result<StructBuilt,Error> {
        let out = match self {
            SettingMode::Set(value) => {
                let tmpl = StructTemplate::new_boolean(*value);
                tmpl.build().map_err(|_| Error::fatal("cannot build booleans!"))
            }
            SettingMode::Member(value, yn) => {                
                let old_tmpl = old.unbuild().map_err(|_| Error::operr("cannot update data"))?;
                Ok(member(&old_tmpl,&StructConst::String(value.to_string()),*yn)
                    .map(|x| x.build())?
                    .map_err(|_| Error::operr("cannot rebuild data"))?)
            },
            SettingMode::None => { Ok(old.clone()) }
        };
        out
    }
}
