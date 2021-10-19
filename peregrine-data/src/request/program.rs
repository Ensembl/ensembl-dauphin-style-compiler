use crate::Channel;
use std::any::Any;
use std::collections::{ HashMap };
use serde_cbor::Value as CborValue;
use crate::util::cbor::{ cbor_array, cbor_string, cbor_map_iter };
use super::backoff::Backoff;
use super::channel::{ PacketPriority };
use super::failure::GeneralFailure;
use super::request::{ RequestType, ResponseType, ResponseBuilderType };
use super::manager::RequestManager;
use crate::run::{ PgDauphin, };
use crate::api::{ PeregrineCoreBase };
use crate::util::message::DataMessage;
use crate::lane::programname::ProgramName;
use crate::util::memoized::{ Memoized, MemoizedType };
use crate::PgCommanderTaskSpec;
use crate::run::add_task;

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
        ProgramCommandRequest {
            program_name: program_name.clone()
        }
    }

    pub(crate) async fn execute(self, manager: &mut RequestManager, dauphin: &PgDauphin) -> Result<(),DataMessage> {
        let mut backoff = Backoff::new(manager,&self.program_name.0,&PacketPriority::RealTime);
        let program_name = self.program_name.clone();
        backoff.backoff::<ProgramCommandResponse,_,_>(self.clone(), move |_| dauphin.is_present(&program_name)).await??;
        if !dauphin.is_present(&self.program_name) {
            return Err(DataMessage::DauphinProgramDidNotLoad(self.program_name));
        }
        Ok(())
    }
}

impl RequestType for ProgramCommandRequest {
    fn type_index(&self) -> u8 { 1 }
    fn serialize(&self, _channel: &Channel) -> Result<CborValue,DataMessage> {
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

async fn load_program(mut base: PeregrineCoreBase, program_name: ProgramName) -> Result<(),DataMessage> {
    let req = ProgramCommandRequest::new(&program_name);
    req.execute(&mut base.manager,&base.dauphin).await
}

fn make_program_loader(base: &PeregrineCoreBase) -> Memoized<ProgramName,Result<(),DataMessage>> {
    let base = base.clone();
    Memoized::new(MemoizedType::Store,move |_,program_name: &ProgramName| {
        let base = base.clone();
        let program_name = program_name.clone();
        Box::pin(async move { load_program(base.clone(),program_name.clone()).await })
    })
}

#[derive(Clone)]
pub struct ProgramLoader(Memoized<ProgramName,Result<(),DataMessage>>);

impl ProgramLoader {
    pub fn new(base: &PeregrineCoreBase) -> ProgramLoader {
        ProgramLoader(make_program_loader(base))
    }

    pub async fn load(&self, program_name: &ProgramName) -> Result<(),DataMessage> {
        self.0.get(program_name).await.as_ref().clone()
    }

    pub fn load_background(&self, base: &PeregrineCoreBase, program_name: &ProgramName) {
        let cache = self.0.clone();
        let program_name = program_name.clone();
        add_task(&base.commander,PgCommanderTaskSpec {
            name: format!("program background loader"),
            prio: 10,
            slot: None,
            timeout: None,
            task: Box::pin(async move {
                cache.get(&program_name).await;
                Ok(())
            }),
            stats: false
        });
    }
}
