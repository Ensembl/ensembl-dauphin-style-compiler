use std::collections::HashMap;
use js_sys::{Function, JsString, Number, Promise };
use peregrine_data::{Stick, StickTopology, StickId, DataRequest, DataRes, BackendNamespace};
use peregrine_toolkit::{error::Error};
use peregrine_toolkit_async::js::promise::promise_to_future;
use wasm_bindgen::JsValue;
use crate::{jsutil::{to_function, to_array, to_string, to_int, to_hashmap, from_map, from_list, emap}, sidecars::JsSidecar, datares::ds_all_datastreams, backend::CallbackError};

// XXX factor into toolkit
fn map_field<'a,X>(data: &'a HashMap<String,X>, field: &str) -> Result<&'a X,CallbackError> {
    emap(data.get(field).ok_or_else(|| Error::operr(&format!("missing field '{}'",field))))
}

async fn do_finish_promise(value: &JsValue) -> Result<JsValue,JsValue> {
    promise_to_future(Promise::resolve(&value).into()).await
}

// TODO toolkit pattern
async fn finish_promise(value: &JsValue) -> Result<JsValue,CallbackError> {
   emap(do_finish_promise(value).await.map_err(|_| Error::operr("bad return from jsapi callback")))
}

#[derive(Clone)]
pub(crate) struct Callbacks {
    track_base: BackendNamespace,
    this: JsValue,
    jump: Option<Function>,
    boot: Option<Function>,
    stickinfo: Option<Function>,
    expansion: Option<Function>,
    program: Option<Function>,
    data: Option<Function>,
}

impl Callbacks {
    pub(crate) fn new(this: JsValue, track_base: &BackendNamespace) -> Callbacks {
        Callbacks {
            this,
            track_base: track_base.clone(),
            jump: None,
            boot: None,
            stickinfo: None,
            expansion: None,
            program: None,
            data: None
        }
    }

    pub(crate) fn add(&mut self, key: &str, value: JsValue) -> Result<(),Error> {
        match key {
            "jump" => { self.jump = Some(to_function(value)?); },
            "boot" => { self.boot = Some(to_function(value)?); },
            "stickinfo" => { self.stickinfo = Some(to_function(value)?); },
            "expand" => { self.expansion = Some(to_function(value)?); },
            "program" => { self.program = Some(to_function(value)?); },
            "data" => { self.data = Some(to_function(value)?); },
            _ => {}
        }
        Ok(())
    }

    fn jump_res(&self, value: &HashMap<String,JsValue>) -> Result<Option<(String,u64,u64)>,CallbackError> {
        Ok(Some((
            to_string(value.get("stick").unwrap())?,
            to_int(map_field(&value,"left")?)? as u64,
            to_int(map_field(&value,"right")?)? as u64
        )))
    }

    pub(crate) async fn jump(&self, location: &str) -> Result<(Option<(String,u64,u64)>,JsSidecar),CallbackError> {
        if let Some(jump) = &self.jump {
            let promise = emap(Error::oper_r(jump.call1(&self.this,&JsString::from(location)),"jump callback"))?;
            let out = finish_promise(&promise).await?;
            let out = to_hashmap(out)?;
            let sidecar = JsSidecar::new_js(&out,&self.track_base)?;
            if !out.contains_key("stick") { return Ok((None,sidecar)); }
            Ok((self.jump_res(&out)?,sidecar))
        } else {
            Ok((None,JsSidecar::new_empty()))
        }
    }

    pub(crate) async fn boot(&self) -> Result<JsSidecar,CallbackError> {
        if let Some(boot) = &self.boot {
            let promise = emap(Error::oper_r(boot.call0(&self.this),"boot callback"))?;
            let out = finish_promise(&promise).await?;
            let out = to_hashmap(out)?;
            let sidecar = JsSidecar::new_js(&out,&self.track_base)?;
            Ok(sidecar)
        } else {
            Ok(JsSidecar::new_empty())
        }
    }

    fn data_args(&self, req: &DataRequest) -> Result<Vec<JsValue>,CallbackError> {
        let region = req.region();
        let scope = from_map(&mut req.scope().iter(),|x| { 
            JsValue::from(from_list(&mut x.iter(), |y| JsString::from(y.clone())))
        })?; /* scope */
        Ok(vec![
            req.name().into(), /* request name */
            region.stick().get_id().into(), /* stick name */
            region.scale().get_index().into(), /* scale */
            region.index().into(), /* index */
            scope.into(), /* scope */
        ])
    }

    pub(crate) async fn data(&self, req: &DataRequest) -> Result<(DataRes,JsSidecar),CallbackError> {
        if let Some(cb) = &self.data {
            let args = from_list(&mut self.data_args(req)?.iter(),|x| x.clone());
            let promise = emap(Error::oper_r(cb.apply(&self.this,&args),"data callback"))?;
            let out = finish_promise(&promise).await?;
            let mut out = to_hashmap(out)?;
            let sidecar = JsSidecar::new_js(&out,&self.track_base)?;
            let invariant = out.get("invariant").map(|x| x.is_truthy()).unwrap_or(false);
            let res = out.remove("data").map(|data| {
                ds_all_datastreams(data)
            }).transpose()?.ok_or_else(|| CallbackError::Internal(Error::operr("missing data")))?;
            Ok((DataRes::new(res,invariant),sidecar))
        } else {
            emap(Err(Error::operr("missing data endpoibnt")))
        }
    }

    pub(crate) async fn expansion(&self, name: &str, step: &str) -> Result<JsSidecar,CallbackError> {
        if let Some(expansion) = &self.expansion {
            let promise = emap(Error::oper_r(expansion.call2(&self.this,&JsString::from(name),&JsString::from(step)),"expansion callback"))?;
            let out = finish_promise(&promise).await?;
            let out = to_hashmap(out)?;
            let sidecar = JsSidecar::new_js(&out,&self.track_base)?;
            Ok(sidecar)
        } else {
            Ok(JsSidecar::new_empty())
        }
    }

    pub(crate) async fn program(&self, group: &str, name: &str, version: u32) -> Result<JsSidecar,CallbackError> {
        if let Some(program) = &self.program {
            let promise = emap(Error::oper_r(program.call3(&self.this,&JsString::from(group),&JsString::from(name),&Number::from(version)),"program callback"))?;
            let out = finish_promise(&promise).await?;
            let out = to_hashmap(out)?;
            let sidecar = JsSidecar::new_js(&out,&self.track_base)?;
            Ok(sidecar)
        } else {
            Ok(JsSidecar::new_empty())
        }
    }

    pub(crate) async fn stickinfo(&self, id: &StickId) -> Result<(Option<Stick>,JsSidecar),CallbackError> {
        if let Some(stick_info) = &self.stickinfo {
            let promise = emap(Error::oper_r(stick_info.call1(&self.this,&JsString::from(id.get_id())),"stick callback"))?;
            let out = finish_promise(&promise).await?;
            let out = to_hashmap(out)?;
            let sidecar = JsSidecar::new_js(&out,&self.track_base)?;
            if !out.contains_key("size") { return Ok((None,sidecar)); }
            let size = to_int(out.get("size").unwrap())?;

            let topology = out.get("topology").map(|topology| {
                emap(StickTopology::from_number(to_int(topology)? as u8))
            }).transpose()?.unwrap_or(StickTopology::Linear);

            let tags = out.get("tags").map(|tags| {
                to_array(tags)
            }).transpose()?.map(|array| {
                array.iter().map(|x| to_string(&x)).collect::<Result<Vec<_>,_>>()
            }).transpose()?.unwrap_or(vec![]);

            Ok((Some(Stick::new(&id,size as u64,topology,&tags)),sidecar))
        } else {
            Ok((None,JsSidecar::new_empty()))
        }
    }
}
