use std::any::Any;
use std::collections::{ HashMap };
use crate::agent::agent::Agent;
use blackbox::blackbox_log;
use serde_cbor::Value as CborValue;
use crate::util::cbor::{ cbor_array, cbor_string, cbor_map_iter };
use crate::util::memoized::{ MemoizedType };
use super::backoff::Backoff;
use super::channel::{ Channel, PacketPriority };
use super::failure::GeneralFailure;
use super::request::{ RequestType, ResponseType, ResponseBuilderType };
use super::manager::RequestManager;
use crate::run::{ PgDauphin, };
use crate::api::{ PeregrineCoreBase, AgentStore };
use crate::util::message::DataMessage;
use crate::lane::programname::ProgramName;

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
    program_name: ProgramName
}

impl ProgramCommandRequest {
    pub(crate) fn new(program_name: &ProgramName) -> ProgramCommandRequest {
        blackbox_log!(&format!("channel-{}",program_name.0.to_string()),"requesting program {}",program_name);
        ProgramCommandRequest {
            program_name: program_name.clone()
        }
    }

    pub(crate) async fn execute(self, manager: &mut RequestManager, dauphin: &PgDauphin) -> Result<(),DataMessage> {
        let mut backoff = Backoff::new();
        let program_name = self.program_name.clone();
        backoff.backoff::<ProgramCommandResponse,_,_>(
            manager,self.clone(),&self.program_name.0,PacketPriority::RealTime, move |_| {
                if dauphin.is_present(&program_name) {
                    None
                } else {
                    Some(GeneralFailure::new("program was returned but did not load successfully"))
                }
            }
        ).await??;
        if !dauphin.is_present(&self.program_name) {
            return Err(DataMessage::DauphinProgramDidNotLoad(self.program_name));
        }
        Ok(())
    }
}

impl RequestType for ProgramCommandRequest {
    fn type_index(&self) -> u8 { 1 }
    fn serialize(&self) -> Result<CborValue,DataMessage> {
        self.program_name.serialize()
    }
    fn to_failure(&self) -> Box<dyn ResponseType> {
        Box::new(GeneralFailure::new("program loading failed"))
    }
}

struct ProgramCommandResponse {}

impl ResponseType for ProgramCommandResponse {
    fn as_any(&self) -> &dyn Any { self }
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
}

pub struct ProgramResponseBuilderType();

impl ResponseBuilderType for ProgramResponseBuilderType {
    fn deserialize(&self, _value: &CborValue) -> anyhow::Result<Box<dyn ResponseType>> {
        Ok(Box::new(ProgramCommandResponse {}))
    }
}

async fn load_program(mut base: PeregrineCoreBase, _agent_store: AgentStore, program_name: ProgramName) -> Result<(),DataMessage> {
    let req = ProgramCommandRequest::new(&program_name);
    req.execute(&mut base.manager,&base.dauphin).await
}

#[derive(Clone)]
pub struct ProgramLoader(Agent<ProgramName,()>);

impl ProgramLoader {
    pub fn new(base: &PeregrineCoreBase, agent_store: &AgentStore) -> ProgramLoader {
        ProgramLoader(Agent::new(MemoizedType::Store,"program-loader",3,base,agent_store, load_program))
    }

    pub async fn load(&self, program_name: &ProgramName) -> Result<(),DataMessage> {
        self.0.get(program_name).await.as_ref().clone()
    }

    pub fn load_background(&self, program_name: &ProgramName) {
        self.0.get_no_wait(program_name)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::{ Arc, Mutex };
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
        let program_name = ProgramName(Channel::new(&ChannelLocation::HttpChannel(urlc(1))),"test2".to_string());
        let pcr = ProgramCommandRequest::new(&program_name);
        let dauphin2 = h.base.dauphin.clone();
        let success = Arc::new(Mutex::new(None));
        let success2 = success.clone();
        let mut manager = h.base.manager.clone();
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
        assert!(h.base.dauphin.is_present(&program_name));
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
        let program_name = ProgramName(Channel::new(&ChannelLocation::HttpChannel(urlc(1))),"test2".to_string());
        let pcr = ProgramCommandRequest::new(&program_name);
        let dauphin2 = h.base.dauphin.clone();
        let success = Arc::new(Mutex::new(None));
        let success2 = success.clone();
        let mut manager = h.base.manager.clone();
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
        assert!(h.base.dauphin.is_present(&program_name));
    }

}