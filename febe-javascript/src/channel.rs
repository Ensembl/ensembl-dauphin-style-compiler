use std::{ pin::Pin, future::Future };
use peregrine_data::{ ChannelSender, BackendNamespace, PacketPriority, MaxiRequest, ChannelMessageDecoder, MaxiResponse, MiniRequest, MiniResponse};
use peregrine_toolkit::{error::Error, log };
use wasm_bindgen::JsValue;
use crate::{payloadextract::PayloadExtractor, backend::Backend, sidecars::JsSidecar};

#[derive(Clone)]
pub struct JavascriptChannel {
    backend_namespace: BackendNamespace,
    backend: Backend
}

impl JavascriptChannel {
    pub(crate) fn new(name: &str, payload: JsValue) -> Result<JavascriptChannel,Error> {
        let backend_namespace = BackendNamespace::new("jsapi",name);
        let payload = PayloadExtractor::new(payload,&backend_namespace)?;

        Ok(JavascriptChannel {
            backend: Backend::new(backend_namespace.clone(),payload.callbacks),
            backend_namespace
        })
    }
    
    pub(crate) fn backend_namespace(&self) -> &BackendNamespace { &&self.backend_namespace }

    async fn send(self, _prio: PacketPriority, maxi: MaxiRequest, _decoder: ChannelMessageDecoder) -> Result<MaxiResponse,Error> {
        let mut out = MaxiResponse::empty(&self.backend_namespace);
        let mut sidecars = JsSidecar::new_empty();
        for attempt in maxi.requests() {
            match attempt.request() {
                MiniRequest::Jump(req) => { 
                    let (res,sidecar) = self.backend.jump(req).await?;
                    out.add_response(attempt.make_response_attempt(MiniResponse::Jump(res)));
                    sidecars.merge(sidecar);
                },
                MiniRequest::BootChannel(req) => {
                    let (res,sidecar) = self.backend.boot(req).await?;
                    out.add_response(attempt.make_response_attempt(MiniResponse::BootChannel(res)));
                    sidecars.merge(sidecar);
                },
                MiniRequest::Stick(req) => {
                    let (res,sidecar) = self.backend.stickinfo(req).await?;
                    out.add_response(attempt.make_response_attempt(MiniResponse::Stick(res)));
                    sidecars.merge(sidecar);
                },
                MiniRequest::Expand(req) => {
                    let (res,sidecar) = self.backend.expansion(req).await?;
                    out.add_response(attempt.make_response_attempt(MiniResponse::Expand(res)));
                    sidecars.merge(sidecar);
                },
                MiniRequest::Program(req) => {
                    let (res,sidecar) = self.backend.program(req).await?;
                    out.add_response(attempt.make_response_attempt(MiniResponse::Program(res)));
                    sidecars.merge(sidecar);
                },
                _ => { 
                    log!("unimplemented");
                    out.add_response(attempt.fail("unimplemented"));
                }
            }
        }
        sidecars.add_to_response(&mut out);
        Ok(out)
    }
}

impl ChannelSender for JavascriptChannel {
    fn get_sender(&self, prio: &PacketPriority, data: MaxiRequest, decoder: ChannelMessageDecoder) -> Pin<Box<dyn Future<Output=Result<MaxiResponse,Error>>>> {
        let self2 = self.clone();
        Box::pin(self2.send(prio.clone(),data,decoder))
    }

    fn backoff(&self) -> bool { false }
}
