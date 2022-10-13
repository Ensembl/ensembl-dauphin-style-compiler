use js_sys::{Function, JsString};
use peregrine_data::{Stick, StickTopology, StickId};
use peregrine_toolkit::error::Error;
use wasm_bindgen::JsValue;
use crate::jsutil::{to_function, to_array, to_string, to_int, to_hashmap};

#[derive(Clone)]
pub(crate) struct Callbacks {
    this: JsValue,
    jump: Option<Function>,
    boot: Option<Function>,
    stickinfo: Option<Function>
}

impl Callbacks {
    pub(crate) fn new(this: Option<JsValue>) -> Callbacks {
        let this = this.unwrap_or(JsValue::NULL);
        Callbacks {
            this,
            jump: None,
            boot: None,
            stickinfo: None
        }
    }

    pub(crate) fn add(&mut self, key: &str, value: JsValue) -> Result<(),Error> {
        match key {
            "jump" => { self.jump = Some(to_function(value)?); },
            "boot" => { self.boot = Some(to_function(value)?); },
            "stickinfo" => { self.stickinfo = Some(to_function(value)?); },
            _ => {}
        }
        Ok(())
    }

    pub(crate) fn jump(&self, location: &str) -> Result<Option<(String,u64,u64)>,Error> {
        if let Some(jump) = &self.jump {
            let out = Error::oper_r(jump.call1(&self.this,&JsString::from(location)),"jump callback")?;
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

    pub(crate) fn boot(&self) -> Result<(),Error> {
        if let Some(boot) = &self.boot {
            let out = Error::oper_r(boot.call0(&self.this),"boot callback")?;
            let _out = to_hashmap(out)?;
        }
        Ok(())
    }

    pub(crate) fn stickinfo(&self, id: &StickId) -> Result<Option<Stick>,Error> {
        if let Some(stick_info) = &self.stickinfo {
            let out = Error::oper_r(stick_info.call1(&self.this,&JsString::from(id.get_id())),"stick callback")?;
            if out.is_null() {
                return Ok(None);
            }
            let mut size = None;
            let mut topology = StickTopology::Linear;
            let mut tags = vec![];
            for (key,value) in to_hashmap(out)? {
                match key.as_str() {
                    "size" => { size = Some(to_int(value)?); },
                    "tags" => { 
                        let array = to_array(value)?;
                        tags = array.iter().map(|x| to_string(x)).collect::<Result<_,_>>()?;
                    },
                    "topology" => {
                        topology = StickTopology::from_number(to_int(value)? as u8)?;
                    },
                    _ => {}
                }
            }
            let size = size.ok_or_else(|| Error::operr("missing size"))?;
            Ok(Some(Stick::new(&id,size as u64,topology,&tags)))
        } else {
            Ok(None)
        }
    }
}
