use std::collections::HashSet;
use std::sync::{ Arc, Mutex };
use std::future::Future;
use std::pin::Pin;
use crate::{ Channel, ChannelIntegration, PacketPriority };
use commander::cdr_timer;
use serde_cbor::Value as CborValue;
#[cfg(test)]
use serde_json::Value as JsonValue;

fn apply_vars(cbor: &CborValue, vars: &[CborValue]) -> CborValue {
    match cbor {
        CborValue::Text(t) => {
            for (i,v) in vars.iter().enumerate() {
                if t == &format!("${}",i) {
                    return v.clone();
                }
            }
            CborValue::Text(t.to_string())
        },
        CborValue::Array(v) => CborValue::Array(v.iter().map(|x| apply_vars(x,vars)).collect()),
        CborValue::Map(m) => CborValue::Map(m.iter().map(|(k,v)| (k.clone(), apply_vars(v,vars))).collect()),
        x => x.clone()
    }
}

fn json_to_cbor(json: &JsonValue) -> CborValue {
    match json {
        JsonValue::Null => CborValue::Null,
        JsonValue::Bool(b) => CborValue::Bool(*b),
        JsonValue::Number(n) => CborValue::Integer(n.as_i64().unwrap() as i128),
        JsonValue::String(s) => CborValue::Text(s.to_string()),
        JsonValue::Array(v) => CborValue::Array(v.iter().map(|x| json_to_cbor(x)).collect()),
        JsonValue::Object(m) => CborValue::Map(m.iter().map(|(k,v)| (CborValue::Text(k.to_string()), json_to_cbor(v))).collect())
    }
}

pub fn cbor_matches(json: &JsonValue, cbor: &CborValue) -> bool {
    match (json,cbor) {
        (JsonValue::String(a),_) if a == "$$" => true,
        (JsonValue::Null,CborValue::Null) => true,
        (JsonValue::Bool(a),CborValue::Bool(b)) => a==b,
        (JsonValue::Number(a),CborValue::Integer(b)) => a.as_i64().unwrap()==*b as i64,
        (JsonValue::String(a),CborValue::Text(b)) => a==b,
        (JsonValue::Array(a),CborValue::Array(b)) => {
            a.iter().zip(b.iter()).map(|(a,b)| { cbor_matches(a,b) }).all(|x| x)
        },
        (JsonValue::Object(a),CborValue::Map(b)) => {
            let mut ak : HashSet<_> = a.keys().collect();
            for k in b.keys() {
                if let CborValue::Text(t) = k {
                    if !ak.remove(t) { return false; /* in b, not a */ }
                    let av = a.get(t).unwrap();
                    let bv = b.get(k).unwrap();
                    if !cbor_matches(&av,&bv) { return false; }
                } else {
                    return false; /* weird key */
                }
            }
            if ak.len() > 0 { return false; /* in a, not b */ }
            true
        },
        _ => false
    }
}

pub fn cbor_matches_print(json: &JsonValue, cbor: &CborValue) -> bool {
    let out = cbor_matches(json,cbor);
    if !out {
        print!("expected:\n{:?}\ngot:\n{:?}\n",json,cbor);
    }
    out
}

#[derive(Clone)]
pub struct TestChannelIntegration {
    timeouts: Arc<Mutex<Vec<(Channel,f64)>>>,
    requests: Arc<Mutex<Vec<CborValue>>>,
    responses: Arc<Mutex<Vec<CborValue>>>,
    wait: Arc<Mutex<f64>>
}

impl TestChannelIntegration {
    pub fn new() -> TestChannelIntegration {
        TestChannelIntegration {
            timeouts: Arc::new(Mutex::new(vec![])),
            requests: Arc::new(Mutex::new(vec![])),
            responses: Arc::new(Mutex::new(vec![])),
            wait: Arc::new(Mutex::new(0.))
        }
    }

    pub fn get_timeouts(&self) -> Vec<(Channel,f64)> {
        self.timeouts.lock().unwrap().drain(..).collect()
    }

    pub fn add_response(&self, data: JsonValue, vars: Vec<CborValue>) {
        self.responses.lock().unwrap().push(apply_vars(&json_to_cbor(&data),&vars));
    }

    pub fn get_requests(&self) -> Vec<CborValue> {
        self.requests.lock().unwrap().drain(..).collect()
    }

    pub fn wait(&mut self, w: f64) { *self.wait.lock().unwrap() = w; }
}

impl ChannelIntegration for TestChannelIntegration {
    fn get_sender(&self, _channel: Channel, _prio: PacketPriority, data: CborValue) -> Pin<Box<dyn Future<Output=Result<CborValue,DataMessage>>>> {
        self.requests.lock().unwrap().push(data);
        let resp = self.responses.clone();
        if resp.lock().unwrap().len() == 0 {
            panic!("unit test didn't provide enough responses!");
        }
        let wait = *self.wait.lock().unwrap();
        Box::pin(async move { 
            if wait > 0. {
                cdr_timer(wait).await;
            }
            Ok(resp.lock().unwrap().remove(0))
        })
    }

    fn set_timeout(&self, channel: &Channel, timeout: f64) {
        self.timeouts.lock().unwrap().push((channel.clone(),timeout));
    }
}
