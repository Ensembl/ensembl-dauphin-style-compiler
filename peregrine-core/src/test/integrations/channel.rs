use std::collections::HashSet;
use std::sync::{ Arc, Mutex };
use std::future::Future;
use std::pin::Pin;
use crate::{ Channel, ChannelIntegration, PacketPriority };
use super::console::TestConsole;
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

#[derive(Clone)]
pub struct TestChannelIntegration {
    console: TestConsole,
    timeouts: Arc<Mutex<Vec<(Channel,f64)>>>,
    requests: Arc<Mutex<Vec<CborValue>>>,
    responses: Arc<Mutex<Vec<CborValue>>>,
}

impl TestChannelIntegration {
    pub fn new(console: &TestConsole) -> TestChannelIntegration {
        TestChannelIntegration {
            console: console.clone(),
            timeouts: Arc::new(Mutex::new(vec![])),
            requests: Arc::new(Mutex::new(vec![])),
            responses: Arc::new(Mutex::new(vec![]))
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
}

impl ChannelIntegration for TestChannelIntegration {
    fn get_sender(&self, _channel: Channel, _prio: PacketPriority, data: CborValue) -> Pin<Box<dyn Future<Output=anyhow::Result<CborValue>>>> {
        self.requests.lock().unwrap().push(data);
        let resp = self.responses.clone();
        if resp.lock().unwrap().len() == 0 {
            panic!("unit test didn't provide enough responses!");
        }
        Box::pin(async move { Ok(resp.lock().unwrap().remove(0)) })
    }

    fn error(&self, _channel: &Channel, msg: &str) {
        self.console.message(msg);
    }

    fn set_timeout(&self, channel: &Channel, timeout: f64) {
        self.timeouts.lock().unwrap().push((channel.clone(),timeout));
    }
}
