use std::collections::HashMap;
use js_sys::{Function, JsString, Number, Promise, Array, Reflect };
use peregrine_data::{Stick, StickTopology, StickId, DataRequest, DataRes, DataAlgorithm, ReceivedData, BackendNamespace};
use peregrine_toolkit::{error::Error};
use peregrine_toolkit_async::js::promise::promise_to_future;
use wasm_bindgen::JsValue;
use crate::{jsutil::{to_function, to_array, to_string, to_int, to_hashmap, from_map, from_list}, sidecars::JsSidecar};

// XXX factor into toolkit
fn map_field<'a,X>(data: &'a HashMap<String,X>, field: &str) -> Result<&'a X,Error> {
    data.get(field).ok_or_else(|| Error::operr(&format!("missing field '{}'",field)))
}

async fn do_finish_promise(value: &JsValue) -> Result<JsValue,JsValue> {
    promise_to_future(Promise::resolve(&value).into()).await
}

// TODO toolkit pattern
async fn finish_promise(value: &JsValue) -> Result<JsValue,Error> {
   do_finish_promise(value).await.map_err(|_| Error::operr("bad return from jsapi callback"))
}

fn array_to_code(value: &Array) -> Result<String,Error> {
    let out = if value.length() == 0 {
        "E"
    } else {
        let first = value.get(0);
        if first.as_f64().is_some() {
            "NRA"
        } else if first.as_bool().is_some() {
            "BA"
        } else if first.is_string() {
            "SA"
        } else {
            return Err(Error::operr("unknown array"));
        }
    };
    Ok(out.to_string())
}

/* Arrays of numbers, strings, bools, get NRA, SA, BA.
 * Objects use "code" and "data" keys to specify directly.
 */
fn add_alg_context(data: JsValue) -> Result<JsValue,Error> {
    let out = Array::new();
    let (code,data) = if Array::is_array(&data) {
        let input = Array::from(&data);
        let code = array_to_code(&input)?;
        (code,input)
    } else {
        let code = to_string(&Reflect::get(&data,&JsValue::from("code")).map_err(|_| Error::operr("missing code key"))?)?;
        let data = Reflect::get(&data,&JsValue::from("data")).map_err(|_| Error::operr("missing data key"))?.into();
        (code,data)
    };
    out.set(0,code.into());
    out.set(1,data.into());
    Ok(out.into())
}

fn ds_one_datastream(v: JsValue) -> Result<ReceivedData,Error> {
    let x : Result<DataAlgorithm,_> = serde_wasm_bindgen::from_value(add_alg_context(v)?);
    let y = x.map_err(|e| Error::operr(&format!("cannot deserialize: {:?}",e)))?;
    y.to_received_data().map_err(|_| Error::operr("cannot deserialize"))
}

fn ds_all_datastreams(data: JsValue) -> Result<HashMap<String,ReceivedData>,Error> {
    to_hashmap(data)?.drain().map(|(k,v)| {
        Ok((k,ds_one_datastream(v)?))
    }).collect()
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

    pub(crate) async fn jump(&self, location: &str) -> Result<(Option<(String,u64,u64)>,JsSidecar),Error> {
        if let Some(jump) = &self.jump {
            let promise = Error::oper_r(jump.call1(&self.this,&JsString::from(location)),"jump callback")?;
            let out = finish_promise(&promise).await?;
            let out = to_hashmap(out)?;
            let sidecar = JsSidecar::new_js(&out,&self.track_base)?;
            if !out.contains_key("stick") { return Ok((None,sidecar)); }
            Ok((Some((
                to_string(out.get("stick").unwrap())?,
                to_int(map_field(&out,"left")?)? as u64,
                to_int(map_field(&out,"right")?)? as u64
            )),sidecar))
        } else {
            Ok((None,JsSidecar::new_empty()))
        }
    }

    pub(crate) async fn boot(&self) -> Result<JsSidecar,Error> {
        if let Some(boot) = &self.boot {
            let promise = Error::oper_r(boot.call0(&self.this),"boot callback")?;
            let out = finish_promise(&promise).await?;
            let out = to_hashmap(out)?;
            let sidecar = JsSidecar::new_js(&out,&self.track_base)?;
            Ok(sidecar)
        } else {
            Ok(JsSidecar::new_empty())
        }
    }

    fn data_args(&self, req: &DataRequest) -> Result<Vec<JsValue>,Error> {
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

    pub(crate) async fn data(&self, req: &DataRequest) -> Result<(DataRes,JsSidecar),Error> {
        if let Some(cb) = &self.data {
            let args = from_list(&mut self.data_args(req)?.iter(),|x| x.clone());
            let promise = Error::oper_r(cb.apply(&self.this,&args),"data callback")?;
            let out = finish_promise(&promise).await?;
            let mut out = to_hashmap(out)?;
            let sidecar = JsSidecar::new_js(&out,&self.track_base)?;
            let invariant = out.get("invariant").map(|x| x.is_truthy()).unwrap_or(false);
            let res = out.remove("data").map(|data| {
                ds_all_datastreams(data)
            }).transpose()?.ok_or_else(|| Error::operr("missing data"))?;
            Ok((DataRes::new(res,invariant),sidecar))
        } else {
            Err(Error::operr("missing data endpoibnt"))
        }
    }

    pub(crate) async fn expansion(&self, name: &str, step: &str) -> Result<JsSidecar,Error> {
        if let Some(expansion) = &self.expansion {
            let promise = Error::oper_r(expansion.call2(&self.this,&JsString::from(name),&JsString::from(step)),"expansion callback")?;
            let out = finish_promise(&promise).await?;
            let out = to_hashmap(out)?;
            let sidecar = JsSidecar::new_js(&out,&self.track_base)?;
            Ok(sidecar)
        } else {
            Ok(JsSidecar::new_empty())
        }
    }

    pub(crate) async fn program(&self, group: &str, name: &str, version: u32) -> Result<JsSidecar,Error> {
        if let Some(program) = &self.program {
            let promise = Error::oper_r(program.call3(&self.this,&JsString::from(group),&JsString::from(name),&Number::from(version)),"program callback")?;
            let out = finish_promise(&promise).await?;
            let out = to_hashmap(out)?;
            let sidecar = JsSidecar::new_js(&out,&self.track_base)?;
            Ok(sidecar)
        } else {
            Ok(JsSidecar::new_empty())
        }
    }

    pub(crate) async fn stickinfo(&self, id: &StickId) -> Result<(Option<Stick>,JsSidecar),Error> {
        if let Some(stick_info) = &self.stickinfo {
            let promise = Error::oper_r(stick_info.call1(&self.this,&JsString::from(id.get_id())),"stick callback")?;
            let out = finish_promise(&promise).await?;
            let out = to_hashmap(out)?;
            let sidecar = JsSidecar::new_js(&out,&self.track_base)?;
            if !out.contains_key("size") { return Ok((None,sidecar)); }
            let size = to_int(out.get("size").unwrap())?;

            let topology = out.get("topology").map(|topology| {
                StickTopology::from_number(to_int(topology)? as u8)
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
