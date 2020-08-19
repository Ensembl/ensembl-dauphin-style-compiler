use std::any::Any;
use anyhow::{ self, anyhow as err, bail };
use blackbox::{ blackbox_count, blackbox_log };
use serde_cbor::Value as CborValue;
use crate::util::cbor::{ cbor_array, cbor_int, cbor_string, cbor_map, cbor_map_iter };
use crate::run::pgcommander::{ PgCommander, PgCommanderTaskSpec };
use crate::run::PgDauphin;
use super::channel::{ Channel, PacketPriority };
use super::manager::RequestManager;
use super::program::ProgramLoader;
use super::request::{ RequestType, ResponseType, ResponseBuilderType, CommandResponse };
use super::packet::{ ResponsePacketBuilderBuilder };
use super::backoff::Backoff;

#[derive(Clone)]
pub struct BootstrapCommandRequest {
    commander: PgCommander,
    dauphin: PgDauphin,
    loader: ProgramLoader,
    channel: Channel
}

impl BootstrapCommandRequest {
    fn new(commander: &PgCommander, dauphin: &PgDauphin, loader: &ProgramLoader, channel: Channel) -> BootstrapCommandRequest {
        BootstrapCommandRequest {
            commander: commander.clone(),
            dauphin: dauphin.clone(),
            loader: loader.clone(),
            channel
        }
    }

    async fn execute(self, mut manager: RequestManager) -> anyhow::Result<()> {
        blackbox_log!(&format!("channel-{}",self.channel.to_string()),"issuing bootstrap request");
        blackbox_count!(&format!("channel-{}",self.channel.to_string()),"bootstrap-request",1);
        let commander = self.commander.clone();
        let dauphin = self.dauphin.clone();
        let loader = self.loader.clone();
        let mut backoff = Backoff::new();
        match backoff.backoff_two_messages::<BootstrapCommandResponse,BootstrapFailure,_>(
                                    &mut manager,self.clone(),&self.channel,PacketPriority::RealTime).await? {
            Ok(b) => {
                blackbox_log!(&format!("channel-{}",self.channel.to_string()),"bootstrap response received");
                blackbox_count!(&format!("channel-{}",self.channel.to_string()),"bootstrap-response-success",1);
                Ok(b.bootstrap(&dauphin,&loader,&commander).await?)
            }
            Err(_) => {
                blackbox_count!(&format!("channel-{}",self.channel.to_string()),"bootstrap-response-fail",1);
                bail!("failed to bootstrap to '{}'. backend missing?",self.channel);
            }
        }
    }
}

impl RequestType for BootstrapCommandRequest {
    fn type_index(&self) -> u8 { 0 }
    fn serialize(&self) -> anyhow::Result<CborValue> { Ok(CborValue::Null) }
    fn to_failure(&self) -> Box<dyn ResponseType> { Box::new(BootstrapFailure{}) }
}

pub struct BootstrapCommandResponse {
    channel: Channel,
    name: String // in-channel name
}

impl ResponseType for BootstrapCommandResponse {
    fn as_any(&self) -> &dyn Any { self }
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
}

impl BootstrapCommandResponse {
    async fn bootstrap(&self, dauphin: &PgDauphin, loader: &ProgramLoader, commander: &PgCommander) -> anyhow::Result<()> {
        let dauphin_prog = dauphin.load_program(loader,&self.channel,&self.name).await?;
        commander.add_task(PgCommanderTaskSpec {
            name: "dauphin: bootstrap".to_string(),
            prio: 2,
            slot: None,
            timeout: None,
            task: Box::pin(dauphin_prog.run())
        });
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

pub struct BootstrapFailure {
}

impl ResponseType for BootstrapFailure {
    fn as_any(&self) -> &dyn Any { self }
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
}

pub struct BootstrapFailureBuilderType();
impl ResponseBuilderType for BootstrapFailureBuilderType {
    fn deserialize(&self, _value: &CborValue) -> anyhow::Result<Box<dyn ResponseType>> {
        Ok(Box::new(BootstrapFailure{}))
    }
}

pub fn bootstrap(requests: &RequestManager, loader: &ProgramLoader, commander: &PgCommander, dauphin: &PgDauphin, channel: Channel) -> anyhow::Result<()> {
    let req = BootstrapCommandRequest::new(commander,dauphin,loader,channel);
    let boot_proc = req.execute(requests.clone());
    commander.add_task(PgCommanderTaskSpec {
        name: "bootstrap".to_string(),
        prio: 4,
        slot: None,
        timeout: None,
        task: Box::pin(boot_proc)
    });
    Ok(())
}

pub(super) fn bootstrap_commands(rspbb: &mut ResponsePacketBuilderBuilder) {
    rspbb.register(0,Box::new(BootstrapResponseBuilderType()));
    rspbb.register(1,Box::new(BootstrapFailureBuilderType()));
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::{ Channel, Commander, ChannelLocation };
    use super::super::program::ProgramLoader;
    use crate::request::bootstrap::bootstrap;
    use crate::test::integrations::{ TestChannelIntegration, TestDauphinIntegration, TestConsole, TestCommander, cbor_matches, test_program };
    use serde_json::json;
    use url::Url;

    fn url(idx: u32) -> Url {
        Url::parse(&(format!("http://a.com/{}",idx))).expect("b")
    }

    #[test]
    fn test_bootstrap() {
        let console = TestConsole::new();
        let chi = TestChannelIntegration::new(&console);
        let di = TestDauphinIntegration::new(&console);
        let dauphin = PgDauphin::new(Box::new(di)).expect("d");
        let cdri = TestCommander::new(&console);
        let comm = PgCommander::new(Box::new(cdri.clone()));
        let rm = RequestManager::new(chi.clone(),&dauphin,&comm);
        let loader = ProgramLoader::new(&comm,&rm).expect("c");
        chi.add_response(json! {
            {
                "responses": [
                    [0,0,[[0,url(2).to_string()],"boot"]]
                ],
                "programs": []
            }
        },vec![]);
        chi.add_response(json! {
            {
                "responses": [
                    [1,2,true]
                ],
                "programs": [
                    ["test","$0",{ "boot": "hello" }]
                ]
            }
        },vec![test_program()]);
        bootstrap(&rm,&loader,&comm,&dauphin,Channel::new(&ChannelLocation::HttpChannel(url(1)))).expect("b");
        for _ in 0..30 {
            cdri.tick();
        }
        let reqs = chi.get_requests();
        assert!(cbor_matches(&json! {
            {
               "requests": [
                   [0,0,null]
               ] 
            }
        },&reqs[0]));
        assert!(cbor_matches(&json! {
            {
               "requests": [
                   [1,1,[[0,url(2).to_string()],"boot"]]
               ] 
            }
        },&reqs[1]));
        let v = console.take_all();
        let v : Vec<_> = v.iter().filter(|x| x.contains("world")).collect();
        assert!(v.len()>0);
    }

    #[test]
    fn test_bootstrap_short() {
        let console = TestConsole::new();
        let chi = TestChannelIntegration::new(&console);
        let di = TestDauphinIntegration::new(&console);
        let dauphin = PgDauphin::new(Box::new(di)).expect("d");
        let cdri = TestCommander::new(&console);
        let comm = PgCommander::new(Box::new(cdri.clone()));
        let rm = RequestManager::new(chi.clone(),&dauphin,&comm);
        let loader = ProgramLoader::new(&comm,&rm).expect("c");
        chi.add_response(json! {
            {
                "responses": [
                    [0,0,[[0,url(1).to_string()],"boot"]]
                ],
                "programs": [
                    ["test","$0",{ "boot": "hello" }]
                ]
            }
        },vec![test_program()]);
        bootstrap(&rm,&loader,&comm,&dauphin,Channel::new(&ChannelLocation::HttpChannel(url(1)))).expect("b");
        for _ in 0..30 {
            cdri.tick();
        }
        let reqs = chi.get_requests();
        assert!(cbor_matches(&json! {
            {
               "requests": [
                   [0,0,null]
               ] 
            }
        },&reqs[0]));
        let v = console.take_all();
        let v : Vec<_> = v.iter().filter(|x| x.contains("world")).collect();
        assert!(v.len()>0);
    }
}