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
use crate::run::{ PgCommander, PgDauphin };
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

    pub(crate) async fn execute(self, manager: &mut RequestManager, dauphin: &PgDauphin) -> anyhow::Result<bool> {
        let mut backoff = Backoff::new();
        let resp = backoff.backoff_one_message::<ProgramCommandResponse,_,_>(
                        manager,self.clone(),&self.channel,PacketPriority::RealTime,|s| s.success).await?;
        Ok(resp.is_ok() && dauphin.is_present(&self.channel,&self.name))
    }
}

impl RequestType for ProgramCommandRequest {
    fn type_index(&self) -> u8 { 1 }
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

async fn load_program(mut manager: RequestManager, dauphin: PgDauphin, channel: Channel, name: String) -> anyhow::Result<bool> {
    let req = ProgramCommandRequest::new(&channel,&name);
    req.execute(&mut manager,&dauphin).await
}

#[derive(Clone)]
pub struct ProgramLoader(Arc<Mutex<ProgramLoaderData>>);

impl ProgramLoader {
    pub fn new(commander: &PgCommander, manager: &RequestManager, dauphin: &PgDauphin) -> anyhow::Result<ProgramLoader> {
        let manager2 = manager.clone();
        let dauphin2 = dauphin.clone();
        let out = ProgramLoader(Arc::new(Mutex::new(ProgramLoaderData {
            single_file: SingleFile::new(commander,move |(channel,name) : &(Channel,String)| {
                let manager = manager2.clone();
                let dauphin = dauphin2.clone();
                PgCommanderTaskSpec {
                    name: format!("program-loader-{}-{}",channel,name),
                    prio: 3,
                    timeout: None,
                    slot: None,
                    task: Box::pin(load_program(manager,dauphin,channel.clone(),name.to_string()))
                }
            })
        })));
        Ok(out)
    }

    pub async fn load(&self, channel: &Channel, name: &str) -> anyhow::Result<bool> {
        self.0.lock().unwrap().single_file.request((channel.clone(),name.to_string())).await
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{ Channel, ChannelLocation };
    use crate::test::helpers::{ TestHelpers, urlc };
    use crate::test::integrations::{ TestChannelIntegration, TestDauphinIntegration, TestConsole, TestCommander, cbor_matches, test_program };
    use serde_json::json;
    use url::Url;

    #[test]
    fn test_program_command() {
        let h = TestHelpers::new();
        h.channel.add_response(json! {
            {
                "responses": [
                    [0,2,true]
                ],
                "programs": [
                    ["test","$0",{ "test2": "hello" }]
                ]
            }
        },vec![test_program()]);
        let pcr = ProgramCommandRequest::new(&Channel::new(&ChannelLocation::HttpChannel(urlc(1))),"test2");
        let dauphin2 = h.dauphin.clone();
        let success = Arc::new(Mutex::new(None));
        let success2 = success.clone();
        let mut manager = h.manager.clone();
        h.task(async move {
            let r = pcr.execute(&mut manager,&dauphin2).await?;
            *success2.lock().unwrap() = Some(r);
            Ok(())
        });
        h.run(30);
        assert!(Some(true) == *success.lock().unwrap());
        let reqs = h.channel.get_requests();
        assert!(cbor_matches(&json! {
            {
               "requests": [
                   [0,1,[[0,urlc(1).to_string()],"test2"]]
               ] 
            }
        },&reqs[0]));
        assert!(h.dauphin.is_present(&Channel::new(&ChannelLocation::HttpChannel(urlc(1))),"test2"));
    }

    #[test]
    fn test_program_force_fail_command() {
        let h = TestHelpers::new();
        h.channel.add_response(json! {
            {
                "responses": [
                    [0,2,true]
                ],
                "programs": [
                    ["test","BAD PROGRAM",{ "test2": "hello" }]
                ]
            }
        },vec![]);
        let pcr = ProgramCommandRequest::new(&Channel::new(&ChannelLocation::HttpChannel(urlc(1))),"test2");
        let dauphin2 = h.dauphin.clone();
        let success = Arc::new(Mutex::new(None));
        let success2 = success.clone();
        let mut manager = h.manager.clone();
        h.task(async move {
            let r = pcr.execute(&mut manager,&dauphin2).await?;
            *success2.lock().unwrap() = Some(r);
            Ok(())
        });
        h.run(30);
        assert_eq!(Some(false),*success.lock().unwrap());
        let reqs = h.channel.get_requests();
        assert!(cbor_matches(&json! {
            {
               "requests": [
                   [0,1,[[0,urlc(1).to_string()],"test2"]]
               ] 
            }
        },&reqs[0]));
        assert!(!h.dauphin.is_present(&Channel::new(&ChannelLocation::HttpChannel(urlc(1))),"test2"));
    }

}