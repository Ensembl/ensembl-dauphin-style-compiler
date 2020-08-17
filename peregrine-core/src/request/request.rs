// TODO tied failures

use anyhow::{ self, Context, anyhow as err };
use std::collections::HashMap;
use std::rc::Rc;
use serde_cbor::Value as CborValue;
use url::Url;
use crate::util::cbor::{ cbor_array, cbor_int, cbor_string, cbor_map, cbor_map_iter };

pub enum Channel {
    HttpChannel(Url)
}

impl Channel {
    pub fn deserialize(value: &CborValue) -> anyhow::Result<Channel> {
        let values = cbor_array(value,2,false)?;
        match cbor_int(&values[0],Some(0))? {
            0 => Ok(Channel::HttpChannel(Url::parse(&cbor_string(&values[1])?).map_err(|e| err!(e.to_string())).context("parsing URL")?)),
            _ => err!("bad channel type in deserialize")
        }
    }
}

pub trait RequestType {
    fn type_index(&self) -> u8;
    fn serialize(&self) -> anyhow::Result<CborValue>;
}

pub struct BootstrapCommandRequest {

}

impl RequestType for BootstrapCommandRequest {
    fn type_index(&self) -> u8 { 0 }
    fn serialize(&self) -> anyhow::Result<CborValue> {
        Ok(CborValue::Null)
    }
}

pub struct ProgramCommandRequest {
    name: String // in-channel name
}

impl RequestType for ProgramCommandRequest {
    fn type_index(&self) -> u8 { 0 }
    fn serialize(&self) -> anyhow::Result<CborValue> {
        Ok(CborValue::Text(self.name.to_string()))
    }
}

pub struct CommandRequest(Box<dyn RequestType>);

impl CommandRequest {
    fn serialize(&self) -> anyhow::Result<CborValue> {
        let typ = self.0.type_index();
        Ok(CborValue::Array(vec![CborValue::Integer(typ as i128),self.0.serialize()?]))
    }
}

pub struct CommandResponse(Box<dyn ResponseType>);

pub struct RequestPacket {
    requests: Vec<CommandRequest>
}

pub trait ResponseType {
    fn execute(&self) -> anyhow::Result<()>;
}

pub trait ResponseBuilderType {
    fn deserialize(&self, value: &CborValue) -> anyhow::Result<Box<dyn ResponseType>>;
}

pub struct BootstrapCommandResponse {
    channel: Channel,
    name: String // in-channel name
}

impl ResponseType for BootstrapCommandResponse {
    fn execute(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

pub struct BootstrapResponseBuilderType();
impl ResponseBuilderType for BootstrapResponseBuilderType {
    fn deserialize(&self, value: &CborValue) -> anyhow::Result<Box<dyn ResponseType>> {
        let values = cbor_array(value,2,false)?;
        Ok(Box::new(BootstrapCommandResponse {
            channel: Channel::deserialize(&values[0])?,
            name: cbor_string(&values[1])?
        }))
    }
}

pub struct ProgramCommandResponse {
}

impl ResponseType for ProgramCommandResponse {
    fn execute(&self) -> anyhow::Result<()> { Ok(()) }
}

pub struct ProgramResponseBuilderType();

impl ResponseBuilderType for ProgramResponseBuilderType {
    fn deserialize(&self, value: &CborValue) -> anyhow::Result<Box<dyn ResponseType>> {
        Ok(Box::new(ProgramCommandResponse {}))
    }
}

pub struct SuppliedBundle{
    bundle_name: String,
    program: CborValue,
    names: HashMap<String,String> // in-channel name -> in-bundle name
}

impl SuppliedBundle {
    pub fn new(value: &CborValue) -> anyhow::Result<SuppliedBundle> {
        let values = cbor_array(value,3,false)?;
        let mut names = HashMap::new();
        for (k,v) in cbor_map_iter(&values[2])? {
            names.insert(cbor_string(k)?,cbor_string(v)?);
        }
        Ok(SuppliedBundle {
            bundle_name: cbor_string(&values[0])?,
            program: values[1].clone(),
            names
        })
    }

    pub(crate) fn bundle_name(&self) -> &str { &self.bundle_name }
    pub(crate) fn program(&self) -> &CborValue { &self.program }
    pub(crate) fn name_map(&self) -> impl Iterator<Item=(&str,&str)> {
        self.names.iter().map(|(x,y)| (x as &str,y as &str))
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
    fn new_packet(&self, value: &CborValue) -> anyhow::Result<ResponsePacket> {
        ResponsePacket::new(value,&self.builders).context("parsing server response")
    }
}

pub struct ResponsePacket {
    channel_identity: String,
    responses: Vec<Result<CommandResponse,String>>,
    programs: Vec<SuppliedBundle>
}

impl ResponsePacket {
    pub(crate) fn channel_identity(&self) -> &str { &self.channel_identity }
    pub(crate) fn programs(&self) -> &[SuppliedBundle] { &self.programs }

    fn deserialize_response(value: &CborValue, builders: &Rc<HashMap<u8,Box<dyn ResponseBuilderType>>>) -> anyhow::Result<CommandResponse> {
        let values = cbor_array(value,2,false)?;
        let builder = builders.get(&(cbor_int(&values[0],Some(255))? as u8)).ok_or(err!("bad response type"))?;
        Ok(CommandResponse(builder.deserialize(&values[1])?))
    }

    pub(crate) fn new(value: &CborValue, builders: &Rc<HashMap<u8,Box<dyn ResponseBuilderType>>>) -> anyhow::Result<ResponsePacket> {
        let values = cbor_map(value,&["id","responses","programs"])?;
        let mut responses = vec![];
        for v in cbor_array(&values[1],0,true)? {
            if let CborValue::Text(error) = v {
                responses.push(Err(error.to_string()));
            } else {
                responses.push(Ok(ResponsePacket::deserialize_response(v,builders)?));
            }
        }
        let programs : anyhow::Result<_> = cbor_array(&values[2],0,true)?.iter().map(|x| SuppliedBundle::new(x)).collect();
        Ok(ResponsePacket {
            channel_identity: cbor_string(&values[0])?,
            responses,
            programs: programs?
        })
    }
}
