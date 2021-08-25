use std::any::Any;
use anyhow::{ self, };
use serde_cbor::Value as CborValue;
use crate::ChannelLocation;
use crate::run::pgcommander::{ PgCommanderTaskSpec };
use crate::run::{ PgDauphin, PgDauphinTaskSpec, add_task };
use super::channel::{ Channel, PacketPriority };
use super::failure::GeneralFailure;
use super::manager::RequestManager;
use super::program::ProgramLoader;
use super::request::{ RequestType, ResponseType, ResponseBuilderType };
use super::backoff::Backoff;
use crate::util::message::DataMessage;
use crate::api::{ PeregrineCoreBase, AgentStore, PeregrineApiQueue, ApiMessage };
use crate::lane::programname::ProgramName;
use crate::util::cbor::{ cbor_map, cbor_string };

#[derive(Clone)]
pub struct BootstrapCommandRequest {
    dauphin: PgDauphin,
    queue: PeregrineApiQueue,
    loader: ProgramLoader,
    channel: Channel
}

impl BootstrapCommandRequest {
    fn new(dauphin: &PgDauphin, queue: &PeregrineApiQueue, loader: &ProgramLoader, channel: Channel) -> BootstrapCommandRequest {
        BootstrapCommandRequest {
            dauphin: dauphin.clone(),
            queue: queue.clone(),
            loader: loader.clone(),
            channel
        }
    }

    async fn execute(self, mut manager: RequestManager) -> Result<(),DataMessage> {
        let dauphin = self.dauphin.clone();
        let loader = self.loader.clone();
        let mut backoff = Backoff::new();
        match backoff.backoff::<BootstrapCommandResponse,_,_>(
                                    &mut manager,self.clone(),&self.channel,PacketPriority::RealTime,|_| None).await? {
            Ok(b) => {
                manager.set_lo_divert(&b.channel_hi,&b.channel_lo);
                b.bootstrap(&dauphin,&self.queue,&loader).await?;
                Ok(())
            }
            Err(e) => {
                Err(DataMessage::BadBootstrapCannotStart(self.channel.clone(),Box::new(e.clone())))
            }
        }
    }
}

impl RequestType for BootstrapCommandRequest {
    fn type_index(&self) -> u8 { 0 }
    fn serialize(&self) -> Result<CborValue,DataMessage> { Ok(CborValue::Null) }
    fn to_failure(&self) -> Box<dyn ResponseType> { Box::new(GeneralFailure::new("bootstrap failed")) }
}

pub struct BootstrapCommandResponse {
    program_name: ProgramName,
    channel_hi: Channel,
    channel_lo: Channel
}

impl ResponseType for BootstrapCommandResponse {
    fn as_any(&self) -> &dyn Any { self }
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
}

impl BootstrapCommandResponse {
    async fn bootstrap(&self, dauphin: &PgDauphin, queue: &PeregrineApiQueue, loader: &ProgramLoader) -> Result<(),DataMessage> {
        dauphin.run_program(loader,PgDauphinTaskSpec {
            prio: 2,
            slot: None,
            timeout: None,
            program_name: self.program_name.clone(),
            payloads: None
        }).await?;
        queue.push(ApiMessage::RegeneraateTrackConfig);
        Ok(())
    }
}

pub struct BootstrapResponseBuilderType();
impl ResponseBuilderType for BootstrapResponseBuilderType {
    fn deserialize(&self, value: &CborValue) -> anyhow::Result<Box<dyn ResponseType>> {
        let values = cbor_map(value,&["boot","hi","lo"])?;
        let channel_hi = Channel::deserialize(&values[1])?;
        let channel_lo = Channel::deserialize(&values[2])?;
        Ok(Box::new(BootstrapCommandResponse {
            program_name: ProgramName::deserialize(&values[0])?,
            channel_hi, channel_lo
        }))
    }
}

pub(crate) fn bootstrap(base: &PeregrineCoreBase, agent_store: &AgentStore, channel: Channel, identity: u64) {
    *base.identity.lock().unwrap() = identity;
    let base2 = base.clone();
    let agent_store = agent_store.clone();
    add_task(&base.commander,PgCommanderTaskSpec {
        name: "bootstrap".to_string(),
        prio: 4,
        slot: None,
        timeout: None,
        task: Box::pin(async move {
            let req = BootstrapCommandRequest::new(&base2.dauphin,&base2.queue,&agent_store.program_loader,channel.clone());
            let r = req.execute(base2.manager.clone()).await;
            let r = r.unwrap_or(());
            base2.booted.unlock();
            Ok(())
        }),
        stats: false
    });
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{ Channel, ChannelLocation };
    use crate::request::bootstrap::bootstrap;
    use crate::test::integrations::{ cbor_matches, cbor_matches_print };
    use serde_json::json;
    use crate::test::helpers::{ TestHelpers, urlc };
    use crate::util::miscpromises::CountingPromise;
    use crate::api::MessageSender;
    use crate::util::cbor::{cbor_string };

    #[test]
    fn test_bootstrap() {
        let h = TestHelpers::new();
        h.channel.add_response(json! {
            {
                "responses": [
                    [0,0,[[0,urlc(2).to_string()],"boot"]]
                ],
                "programs": []
            }
        },vec![]);
        h.channel.add_response(json! {
            {
                "responses": [
                    [1,2,true]
                ],
                "programs": [
                    ["test","ok",{ "boot": "hello" }]
                ]
            }
        },vec![]);
        let booted = CountingPromise::new();
        let messages = MessageSender::new(|w| {});
        bootstrap(&h.base,&h.agent_store,Channel::new(&ChannelLocation::HttpChannel(urlc(1))),0);
        h.run(30);
        let reqs = h.channel.get_requests();
        assert!(cbor_matches(&json! {
            {
                "channel": "$$",
               "requests": [
                   [0,0,null]
               ] 
            }
        },&reqs[0]));
        assert!(cbor_matches(&json! {
            {
                "channel": "$$",
               "requests": [
                   [1,1,[[0,urlc(2).to_string()],"boot"]]
               ] 
            }
        },&reqs[1]));
        let d = h.fdr.take_loads();
        assert_eq!(1,d.len());
        assert_eq!("ok",cbor_string(&d[0].data).expect("a"));
    }

    #[test]
    fn test_bootstrap_short() {
        let h = TestHelpers::new();
        h.channel.add_response(json! {
            {
                "responses": [
                    [0,0,[[0,urlc(1).to_string()],"boot"]]
                ],
                "programs": [
                    ["test","ok",{ "boot": "hello" }]
                ]
            }
        },vec![]);
        let booted = CountingPromise::new();
        let messages = MessageSender::new(|w| {});
        bootstrap(&h.base,&h.agent_store,Channel::new(&ChannelLocation::HttpChannel(urlc(1))),0);
        h.run(30);
        let reqs = h.channel.get_requests();
        print!("{:?}\n",reqs[0]);
        assert!(cbor_matches(&json! {
            {
                "channel": "$$",
                "requests": [
                   [0,0,null]
               ] 
            }
        },&reqs[0]));
        let d = h.fdr.take_loads();
        assert_eq!(1,d.len());
        assert_eq!("ok",cbor_string(&d[0].data).expect("a"));
    }

    #[test]
    fn test_temporary_failure() {
        let h = TestHelpers::new();
        h.channel.add_response(json! { "nonsense" },vec![]);
        h.channel.add_response(json! { "nonsense" },vec![]);
        h.channel.add_response(json! {
            {
                "responses": [
                    [2,0,[[0,urlc(1).to_string()],"boot"]]
                ],
                "programs": [
                    ["test","ok",{ "boot": "hello" }]
                ]
            }
        },vec![]);
        let booted = CountingPromise::new();
        let messages = MessageSender::new(|w| {});
        bootstrap(&h.base,&h.agent_store,Channel::new(&ChannelLocation::HttpChannel(urlc(1))),0);
        for _ in 0..5 {
            h.run(30);
            h.commander_inner.add_time(100.);
        }
        let reqs = h.channel.get_requests();
        for i in 0..2 {
            assert!(cbor_matches(&json! {
                {
                    "channel": "$$",
                "requests": [
                    [i,0,null]
                ] 
                }
            },&reqs[i]));
        }
        let d = h.fdr.take_loads();
        assert_eq!(1,d.len());
        assert_eq!("ok",cbor_string(&d[0].data).expect("a"))
    }

    #[test]
    fn test_permanent_failure() {
        let h = TestHelpers::new();
        for _ in 0..100 {
            h.channel.add_response(json! { "nonsense" },vec![]);
        }
        let booted = CountingPromise::new();
        let messages = MessageSender::new(|w| {});
        bootstrap(&h.base,&h.agent_store,Channel::new(&ChannelLocation::HttpChannel(urlc(1))),0);
        for _ in 0..25 {
            h.run(10);
            h.commander_inner.add_time(10000.);
        }
        let reqs = h.channel.get_requests();
        for i in 0..2 {
            assert!(cbor_matches_print(&json! {
                {
                    "channel": "$$",
                "requests": [
                    [i,0,null]
                ]
                }
            },&reqs[i]));
        }
        let v = h.console.lock().unwrap().drain(..).collect::<Vec<_>>();
        print!("{}",v.join("\n"));
        let v : Vec<_> = v.iter().filter(|x| x.contains("Fatal")).collect();
        assert!(v.len()>0);
    }

    #[test]
    fn timeout() {
        let mut h = TestHelpers::new();
        let channel = Channel::new(&ChannelLocation::HttpChannel(urlc(1)));
        h.base.manager.set_timeout(&channel,&PacketPriority::RealTime,42.).expect("a");
        assert_eq!(vec![(channel,42.)],h.channel.get_timeouts());
        h.channel.wait(100.);
        for _ in 0..20 {
            h.channel.add_response(json! {
                {
                    "responses": [
                        [2,0,[[0,urlc(1).to_string()],"boot"]]
                    ],
                    "programs": [
                        ["test","ok",{ "boot": "hello" }]
                    ]
                }
            },vec![]);
        }
        let booted = CountingPromise::new();
        let messages = MessageSender::new(|w| {});
        bootstrap(&h.base,&h.agent_store,Channel::new(&ChannelLocation::HttpChannel(urlc(1))),0);
        for _ in 0..50 {
            h.run(10);
            h.commander_inner.add_time(1000.);
        }
        let v = h.console.lock().unwrap().drain(..).collect::<Vec<_>>();
        print!("v={:?}\n",v);
        let w : Vec<_> = v.iter().filter(|x| x.contains("Fatal")).collect();
        assert!(w.len()>0);
        let w : Vec<_> = v.iter().filter(|x| x.contains("Timeout")).collect();
        assert!(w.len()>0);
    }
}