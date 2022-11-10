use std::{collections::HashMap, mem};
use js_sys::Array;
use peregrine_data::{TrackModel, ExpansionModel, MaxiResponse};
use peregrine_toolkit::{error::Error };
use wasm_bindgen::JsValue;
use serde::Deserialize;

fn js_array_extract<T>(value: &JsValue) -> Result<Vec<T>,Error> where for<'de> T: Deserialize<'de> {
    if !Array::is_array(value) { return Err(Error::operr("expected array")) }
    let value = Array::from(value);
    let value = value.iter().map(|x| {
        serde_wasm_bindgen::from_value(x)
    }).collect::<Result<Vec<_>,_>>();
    match value {
        Ok(x) => Ok(x),
        Err(e) => Err(Error::operr(&format!("cannot deserialize: {}",e)))
    }
}

pub(crate) struct JsSidecar {
    tracks: Vec<TrackModel>,
    expansions: Vec<ExpansionModel>
}

impl JsSidecar {
    pub(crate) fn new_empty() -> JsSidecar {
        JsSidecar { tracks: vec![], expansions: vec![] }
    }

    pub(crate) fn new_js(data: &HashMap<String,JsValue>) -> Result<JsSidecar,Error> {
        let expansions = data.get("expansions").map(|x| {
            js_array_extract(x)
        }).transpose()?.unwrap_or(vec![]);
        let tracks = data.get("tracks").map(|x| {
            js_array_extract(x)
        }).transpose()?.unwrap_or(vec![]);
        Ok(JsSidecar { expansions, tracks })
    }
    
    pub(crate) fn merge(&mut self, mut other: JsSidecar) {
        self.tracks.append(&mut other.tracks);
        self.expansions.append(&mut other.expansions);
    }

    pub(crate) fn add_to_response(&mut self, res: &mut MaxiResponse) {
        res.set_track_payload(
            mem::replace(&mut self.tracks,vec![]),
            mem::replace(&mut self.expansions,vec![]));
    }
}
