use std::collections::BTreeMap;
use peregrine_data::{MaxiRequest, MiniRequest, MiniRequestAttempt};
use serde_cbor::Value as CborValue;

fn make_type_index(request: &MiniRequest) -> u8 { // XXX not pub crate
    match request {
        MiniRequest::BootChannel(_) => 0,
        MiniRequest::Program(_) => 1,
        MiniRequest::Stick(_) => 2,
        MiniRequest::Authority(_) => 3,
        MiniRequest::Data(_) => 4,
        MiniRequest::Jump(_) => 5,
        MiniRequest::Metric(_) => 6
    }
}

fn make_encode(request: &MiniRequest, msgid: u64) -> CborValue {
    CborValue::Array(vec![
        CborValue::Integer(msgid as i128),
        CborValue::Integer(make_type_index(request) as i128),
        make_encode_data(request)
    ])
}

fn make_encode_data(request: &MiniRequest) -> CborValue {
    match request {
        MiniRequest::BootChannel(x) => x.encode(),
        MiniRequest::Program(x) => x.encode(),
        MiniRequest::Stick(x) => x.encode(),
        MiniRequest::Authority(x) => x.encode(),
        MiniRequest::Data(x) => x.encode(),
        MiniRequest::Jump(x) => x.encode(),
        MiniRequest::Metric(x) => x.encode()
    }
}

fn minireq_encode_cbor(request: &MiniRequestAttempt) -> CborValue {
    make_encode(request.request(),request.msgid())
}

pub fn maxireq_encode_cbor(request: &MaxiRequest) -> CborValue {
    let mut map = BTreeMap::new();
    map.insert(CborValue::Text("channel".to_string()), request.channel().encode());
    let requests = request.requests().iter().map(|r| {
        minireq_encode_cbor(r)
    }).collect::<Vec<_>>();  // XXX to take, ie destroy
    map.insert(CborValue::Text("requests".to_string()),CborValue::Array(requests));
    map.insert(CborValue::Text("version".to_string()),request.metadata().encode());
    CborValue::Map(map)
}
