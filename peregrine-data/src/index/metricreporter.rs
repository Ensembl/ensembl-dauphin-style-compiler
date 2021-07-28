use peregrine_message::PeregrineMessage;
use crate::{Channel, PacketPriority, PgCommander, PgCommanderTaskSpec, RequestManager, add_task, request::{failure::GeneralFailure, request::RequestType}};
use serde_cbor::Value as CborValue;
use crate::PeregrineCoreBase;

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct MetricReport {
    identity: u64,
    text: String,
    code: (u64,u64)
}

impl MetricReport {
    pub fn new_from_message(base: &PeregrineCoreBase, message: &(dyn PeregrineMessage + 'static)) -> MetricReport {
        let identity = *base.identity.lock().unwrap();
        MetricReport {
            identity,
            text: message.to_string(),
            code: message.code()
        }
    }

    async fn send_task(&self, mut manager: RequestManager, channel: Channel) {
        // We don't care about errors here: avoid loops and spew
        manager.execute(channel,PacketPriority::Batch,Box::new(self.clone())).await.ok();
    }

    pub(crate) fn send(&self, commander: &PgCommander, manager: &mut RequestManager, channel: &Channel) {
        let self2 = self.clone();
        let manager = manager.clone();
        let channel = channel.clone();
        add_task(commander,PgCommanderTaskSpec {
            name: "message".to_string(),
            prio: 11,
            timeout: None,
            slot: None,
            task: Box::pin(async move { 
                self2.send_task(manager,channel).await;
                Ok(())
            }),
            stats: false
        });

    }
}

impl RequestType for MetricReport {
    fn type_index(&self) -> u8 { 6 }

    fn serialize(&self) -> Result<serde_cbor::Value,crate::DataMessage> {
        Ok(CborValue::Array(vec![
            CborValue::Integer(self.identity as i128),
            CborValue::Text(self.text.clone()),
            CborValue::Integer(self.code.0 as i128),CborValue::Integer(self.code.1 as i128)
        ]))
    }

    fn to_failure(&self) -> Box<dyn crate::request::request::ResponseType> {
        Box::new(GeneralFailure::new("metric reporting failed"))
    }
}