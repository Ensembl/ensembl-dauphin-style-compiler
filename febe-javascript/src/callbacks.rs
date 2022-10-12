use js_sys::{Function, JsString};
use peregrine_toolkit::error::Error;
use wasm_bindgen::JsValue;
use crate::jsutil::{to_function, to_array, to_string, to_int};

#[derive(Clone)]
pub(crate) struct Callbacks {
    this: JsValue,
    jump: Option<Function>
}

impl Callbacks {
    pub(crate) fn new(this: Option<JsValue>) -> Callbacks {
        let this = this.unwrap_or(JsValue::NULL);
        Callbacks {
            this,
            jump: None
        }
    }

    pub(crate) fn add(&mut self, key: &str, value: JsValue) -> Result<(),Error> {
        match key {
            "jump" => { self.jump = Some(to_function(value)?); },
            _ => {}
        }
        Ok(())
    }

    pub(crate) fn jump(&self, location: &str) -> Result<Option<(String,u64,u64)>,Error> {
        if let Some(jump) = &self.jump {
            let out = Error::oper_r(jump.call1(&self.this,&JsString::from(location)),"jump callback return")?;
            if out.is_null() { return Ok(None); }
            let out = to_array(out)?;
            Ok(Some((
                to_string(out.get(0))?,
                to_int(out.get(1))? as u64,
                to_int(out.get(2))? as u64
            )))
        } else {
            Ok(None)
        }
    }
}
