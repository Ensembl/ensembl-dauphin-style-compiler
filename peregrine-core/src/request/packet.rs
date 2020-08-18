use anyhow::{ self, Context, anyhow as err, bail };
use commander::cdr_timer;
use std::collections::{ BTreeMap, HashMap };
use std::mem::replace;
use std::rc::Rc;
use serde_cbor::Value as CborValue;
use super::request::{ RequestType, ResponseType, ResponseBuilderType, CommandResponse, CommandRequest };
use super::program::SuppliedBundle;
use crate::util::cbor::{ cbor_array, cbor_int, cbor_string, cbor_map, cbor_map_iter };


pub struct RequestPacket {
    requests: Vec<CommandRequest>
}

impl RequestPacket {
    pub fn new() -> RequestPacket {
        RequestPacket {
            requests: vec![]
        }
    }

    pub fn add(&mut self, request: CommandRequest) {
        self.requests.push(request);
    }

    pub fn serialize(&self) -> anyhow::Result<CborValue> {
        let mut map = BTreeMap::new();
        let mut requests = vec![];
        for r in &self.requests {
            requests.push(r.serialize()?);
        }
        map.insert(CborValue::Text("requests".to_string()),CborValue::Array(requests));
        Ok(CborValue::Map(map))
    }
}

pub struct ResponsePacketBuilderBuilder {
    builders: HashMap<u8,Box<dyn ResponseBuilderType>>
}

pub struct ResponsePacketBuilder {
    builders: Rc<HashMap<u8,Box<dyn ResponseBuilderType>>>
}

impl ResponsePacketBuilderBuilder {
    pub fn new() -> ResponsePacketBuilderBuilder {
        ResponsePacketBuilderBuilder {
            builders: HashMap::new()
        }
    }

    pub fn register(&mut self, type_index: u8, rbt: Box<dyn ResponseBuilderType>) {
        self.builders.insert(type_index,rbt);
    }

    pub fn build(self) -> ResponsePacketBuilder {
        ResponsePacketBuilder {
            builders: Rc::new(self.builders)
        }
    }
}

impl ResponsePacketBuilder {
    pub fn new_packet(&self, value: &CborValue) -> anyhow::Result<ResponsePacket> {
        ResponsePacket::new(value,&self.builders).context("parsing server response")
    }
}

pub struct ResponsePacket {
    channel_identity: String,
    responses: Vec<CommandResponse>,
    programs: Vec<SuppliedBundle>
}

impl ResponsePacket {
    pub(crate) fn channel_identity(&self) -> &str { &self.channel_identity }
    pub(crate) fn programs(&self) -> &[SuppliedBundle] { &self.programs }
    pub(crate) fn responses(&self) -> &[CommandResponse] { &self.responses }
    pub(crate) fn take_responses(&mut self) -> Vec<CommandResponse> {
        replace(&mut self.responses,vec![])
    }

    fn deserialize_response(value: &CborValue, builders: &Rc<HashMap<u8,Box<dyn ResponseBuilderType>>>) -> anyhow::Result<CommandResponse> {
        let values = cbor_array(value,3,false)?;
        let msgid = cbor_int(&values[0],None)? as u64;
        let builder = builders.get(&(cbor_int(&values[1],Some(255))? as u8)).ok_or(err!("bad response type"))?;
        Ok(CommandResponse::new(msgid,builder.deserialize(&values[2])?))
    }

    fn new(value: &CborValue, builders: &Rc<HashMap<u8,Box<dyn ResponseBuilderType>>>) -> anyhow::Result<ResponsePacket> {
        let values = cbor_map(value,&["id","responses","programs"])?;
        let mut responses = vec![];
        for v in cbor_array(&values[1],0,true)? {
            responses.push(ResponsePacket::deserialize_response(v,builders)?);
        }
        let programs : anyhow::Result<_> = cbor_array(&values[2],0,true)?.iter().map(|x| SuppliedBundle::new(x)).collect();
        Ok(ResponsePacket {
            channel_identity: cbor_string(&values[0])?,
            responses,
            programs: programs?
        })
    }
}
