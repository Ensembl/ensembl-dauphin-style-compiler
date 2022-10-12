use std::{ pin::Pin, future::Future };
use peregrine_data::{ ChannelSender, BackendNamespace, PacketPriority, MaxiRequest, ChannelMessageDecoder, MaxiResponse, MiniRequest, MiniResponse};
use peregrine_toolkit::{error::Error, log };
use wasm_bindgen::JsValue;
use crate::{payloadextract::PayloadExtractor, backend::Backend};

#[derive(Clone)]
pub struct JavascriptChannel {
    backend_namespace: BackendNamespace,
    backend: Backend
}

impl JavascriptChannel {
    pub(crate) fn new(name: &str, payload: JsValue) -> Result<JavascriptChannel,Error> {
        let payload = PayloadExtractor::new(payload)?;
        Ok(JavascriptChannel {
            backend_namespace: BackendNamespace::new("jsapi",name),
            backend: Backend::new(payload.callbacks)
        })
    }
    
    pub(crate) fn backend_namespace(&self) -> &BackendNamespace { &&self.backend_namespace }

    async fn send(self, _prio: PacketPriority, maxi: MaxiRequest, _decoder: ChannelMessageDecoder) -> Result<MaxiResponse,Error> {
        let mut out = MaxiResponse::empty(&self.backend_namespace);
        for attempt in maxi.requests() {
            match attempt.request() {
                MiniRequest::Jump(req) => { 
                    let res = self.backend.jump(req)?;
                    out.add_response(attempt.make_response_attempt(MiniResponse::Jump(res)));
                },
                _ => { 
                    log!("unimplemented");
                    out.add_response(attempt.fail("unimplemented"));
                }
            }
        }
        Ok(out)
    }
}

impl ChannelSender for JavascriptChannel {
    fn get_sender(&self, prio: &PacketPriority, data: MaxiRequest, decoder: ChannelMessageDecoder) -> Pin<Box<dyn Future<Output=Result<MaxiResponse,Error>>>> {
        let self2 = self.clone();
        Box::pin(self2.send(prio.clone(),data,decoder))
    }
}
