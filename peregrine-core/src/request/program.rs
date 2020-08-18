use std::any::Any;
use std::collections::{ HashMap, HashSet };
use std::sync::{ Arc, Mutex };
use anyhow::{ bail };
use blackbox::blackbox_log;
use serde_cbor::Value as CborValue;
use crate::util::cbor::{ cbor_array, cbor_bool, cbor_string, cbor_map, cbor_map_iter };
use crate::util::singlefile::SingleFile;
use super::backoff::Backoff;
use super::channel::{ Channel, PacketPriority };
use super::packet::ResponsePacketBuilderBuilder;
use super::request::{ RequestType, ResponseType, ResponseBuilderType, CommandResponse };
use super::manager::RequestManager;
use crate::run::{ PgCommander };
use crate::run::pgcommander::PgCommanderTaskSpec;

pub struct SuppliedBundle {
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

#[derive(Clone)]
struct ProgramCommandRequest {
    channel: Channel,
    name: String // in-channel name
}

impl ProgramCommandRequest {
    pub(crate) fn new(channel: &Channel, name: &str) -> ProgramCommandRequest {
        blackbox_log!(&format!("channel-{}",self.channel.to_string()),"requesting program {}",name);
        ProgramCommandRequest {
            channel: channel.clone(),
            name: name.to_string()
        }
    }

    pub(crate) async fn execute(self, manager: &mut RequestManager) -> anyhow::Result<bool> {
        let mut backoff = Backoff::new();
        let resp = backoff.backoff_one_message::<ProgramCommandResponse,_,_>(
                        manager,self.clone(),&self.channel,PacketPriority::RealTime,|s| s.success).await?;
        Ok(resp.is_ok())
    }
}

impl RequestType for ProgramCommandRequest {
    fn type_index(&self) -> u8 { 0 }
    fn serialize(&self) -> anyhow::Result<CborValue> {
        Ok(CborValue::Array(vec![self.channel.serialize()?,CborValue::Text(self.name.to_string())]))
    }
    fn to_failure(&self) -> Box<dyn ResponseType> {
        Box::new(ProgramCommandResponse{ success: false })
    }
}

struct ProgramCommandResponse {
    success: bool
}

impl ResponseType for ProgramCommandResponse {
    fn as_any(&self) -> &dyn Any { self }
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
}

pub struct ProgramResponseBuilderType();

impl ResponseBuilderType for ProgramResponseBuilderType {
    fn deserialize(&self, value: &CborValue) -> anyhow::Result<Box<dyn ResponseType>> {
        Ok(Box::new(ProgramCommandResponse {
            success: cbor_bool(value)?
        }))
    }
}

pub(super) fn program_commands(rspbb: &mut ResponsePacketBuilderBuilder) {
    rspbb.register(2,Box::new(ProgramResponseBuilderType()));
}

struct ProgramLoaderData {
    single_file: SingleFile<(Channel,String),bool>
}

async fn load_program(mut manager: RequestManager, channel: Channel, name: String) -> anyhow::Result<bool> {
    let req = ProgramCommandRequest::new(&channel,&name);
    req.execute(&mut manager).await
}

#[derive(Clone)]
pub struct ProgramLoader(Arc<Mutex<ProgramLoaderData>>);

impl ProgramLoader {
    pub fn new(commander: &PgCommander, manager: &RequestManager) -> anyhow::Result<ProgramLoader> {
        let manager2 = manager.clone();
        let out = ProgramLoader(Arc::new(Mutex::new(ProgramLoaderData {
            single_file: SingleFile::new(commander,move |(channel,name) : &(Channel,String)| {
                let manager = manager2.clone();
                PgCommanderTaskSpec {
                    name: format!("program-loader-{}-{}",channel,name),
                    prio: 3,
                    timeout: None,
                    slot: None,
                    task: Box::pin(load_program(manager,channel.clone(),name.to_string()))
                }
            })
        })));
        Ok(out)
    }

    pub async fn load(&self, channel: &Channel, name: &str) -> anyhow::Result<bool> {
        self.0.lock().unwrap().single_file.request((channel.clone(),name.to_string())).await
    }
}
