use crate::lock;
use anyhow::bail;
use std::any::Any;
use std::collections::{ HashMap };
use std::rc::Rc;
use std::sync::{ Arc, Mutex };
use blackbox::blackbox_log;
use serde_cbor::Value as CborValue;
use crate::util::cbor::{ cbor_array, cbor_string, cbor_map_iter };
use crate::util::memoized::Memoized;
use super::backoff::Backoff;
use super::channel::{ Channel, PacketPriority };
use super::failure::GeneralFailure;
use super::request::{ RequestType, ResponseType, ResponseBuilderType };
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
        blackbox_log!(&format!("channel-{}",channel.to_string()),"requesting program {}",name);
        ProgramCommandRequest {
            channel: channel.clone(),
            name: name.to_string()
        }
    }

    pub(crate) async fn execute(self, manager: &mut RequestManager, dauphin: &PgDauphin) -> anyhow::Result<()> {
        let mut backoff = Backoff::new();
        let channel = self.channel.clone();
        let name = self.name.clone();
        backoff.backoff::<ProgramCommandResponse,_,_>(
            manager,self.clone(),&self.channel,PacketPriority::RealTime, move |_| {
                if dauphin.is_present(&channel,&name) {
                    None
                } else {
                    Some(GeneralFailure::new("program was returned but did not load successfully"))
                }
            }
        ).await??;
        if !dauphin.is_present(&self.channel,&self.name) {
            bail!("program did not load");
        }
        Ok(())
    }
}

impl RequestType for ProgramCommandRequest {
    fn type_index(&self) -> u8 { 1 }
    fn serialize(&self) -> anyhow::Result<CborValue> {
        Ok(CborValue::Array(vec![self.channel.serialize()?,CborValue::Text(self.name.to_string())]))
    }
    fn to_failure(&self) -> Rc<dyn ResponseType> {
        Rc::new(GeneralFailure::new("program loading failed"))
    }
}

struct ProgramCommandResponse {}

impl ResponseType for ProgramCommandResponse {
    fn as_any(&self) -> &dyn Any { self }
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
}

pub struct ProgramResponseBuilderType();

impl ResponseBuilderType for ProgramResponseBuilderType {
    fn deserialize(&self, _value: &CborValue) -> anyhow::Result<Rc<dyn ResponseType>> {
        Ok(Rc::new(ProgramCommandResponse {}))
    }
}

async fn load_program(mut manager: RequestManager, dauphin: PgDauphin, channel: Channel, name: String) -> anyhow::Result<()> {
    let req = ProgramCommandRequest::new(&channel,&name);
    req.execute(&mut manager,&dauphin).await
}

#[derive(Clone)]
pub struct ProgramLoader {
    store: Memoized<(Channel,String),()>,
    manager: RequestManager
}

impl ProgramLoader {
    pub fn new(commander: &PgCommander, manager: &RequestManager, dauphin: &PgDauphin) -> ProgramLoader {
        let manager = manager.clone();
        let dauphin = dauphin.clone();
        let commander = commander.clone();
        ProgramLoader {
            manager: manager.clone(),
            store: Memoized::new(move |key: &(Channel,String),result| {
                let (channel,name) = (key.0.clone(),key.1.clone());
                let manager = manager.clone();
                let dauphin = dauphin.clone();
                commander.add_task(PgCommanderTaskSpec {
                    name: format!("program-loader-{}-{}",channel,name),
                    prio: 3,
                    timeout: None,
                    slot: None,
                    task: Box::pin(async move {
                        load_program(manager,dauphin,channel,name).await.unwrap_or(());
                        result.resolve(());
                        Ok(())
                    })
                })
            })
        }
    }

    pub async fn load(&self, channel: &Channel, name: &str) -> anyhow::Result<()> {
        self.store.get(&(channel.clone(),name.to_string())).await.unwrap_or(Arc::new(()));
        Ok(())
    }

    pub fn load_background(&self, channel: &Channel, name: &str) -> anyhow::Result<()> {
        self.manager.execute_background(channel,Box::new(ProgramCommandRequest::new(channel,name)))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{ Channel, ChannelLocation };
    use crate::test::helpers::{ TestHelpers, urlc };
    use crate::test::integrations::{ cbor_matches };
    use serde_json::json;

    #[test]
    fn test_program_command() {
        let h = TestHelpers::new();
        h.channel.add_response(json! {
            {
                "responses": [
                    [0,2,true]
                ],
                "programs": [
                    ["test","ok",{ "test2": "hello" }]
                ],
            }
        },vec![]);
        let pcr = ProgramCommandRequest::new(&Channel::new(&ChannelLocation::HttpChannel(urlc(1))),"test2");
        let dauphin2 = h.dauphin.clone();
        let success = Arc::new(Mutex::new(None));
        let success2 = success.clone();
        let mut manager = h.manager.clone();
        h.task(async move {
            let r = pcr.execute(&mut manager,&dauphin2).await;
            *success2.lock().unwrap() = Some(r.is_ok());
            Ok(())
        });
        h.run(30);
        assert!(Some(true) == *success.lock().unwrap());
        let reqs = h.channel.get_requests();
        assert!(cbor_matches(&json! {
            {
                "channel": "$$",
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
        for i in 0..20 {
            h.channel.add_response(json! {
                {
                    "responses": [
                        [i,2,true]
                    ],
                    "programs": [
                        ["test","BAD PROGRAM",{ "test2": "hello" }]
                    ]
                }
            },vec![]);
        }
        let pcr = ProgramCommandRequest::new(&Channel::new(&ChannelLocation::HttpChannel(urlc(1))),"test2");
        let dauphin2 = h.dauphin.clone();
        let success = Arc::new(Mutex::new(None));
        let success2 = success.clone();
        let mut manager = h.manager.clone();
        h.task(async move {
            let r = pcr.execute(&mut manager,&dauphin2).await;
            *success2.lock().unwrap() = Some(r.is_ok());
            Ok(())
        });
        for _ in 0..2000 {
            h.run(30);
            h.commander_inner.add_time(100.);
        }
        assert_eq!(false,success.lock().unwrap().unwrap());
        let reqs = h.channel.get_requests();
        assert!(cbor_matches(&json! {
            {
                "channel": "$$",
               "requests": [
                   [0,1,[[0,urlc(1).to_string()],"test2"]]
               ] 
            }
        },&reqs[0]));
        assert!(!h.dauphin.is_present(&Channel::new(&ChannelLocation::HttpChannel(urlc(1))),"test2"));
    }

}