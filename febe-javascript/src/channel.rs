use std::{ pin::Pin, future::Future };
use peregrine_data::{ ChannelSender, BackendNamespace, PacketPriority, MaxiRequest, ChannelMessageDecoder, MaxiResponse, MiniRequest, MiniResponse, FailureRes};
use peregrine_toolkit::{error::Error };
use wasm_bindgen::JsValue;
use crate::{payloadextract::PayloadExtractor, backend::{Backend, CallbackError}, sidecars::JsSidecar};

fn map_error<F,X>(input: Result<(X,JsSidecar),CallbackError>, cb: F) -> Result<(MiniResponse,JsSidecar),Error>
        where F: FnOnce(X) -> MiniResponse {
    match input {
        Err(CallbackError::External(value)) => Ok((MiniResponse::FailureRes(FailureRes::new(&value)),JsSidecar::new_empty())),
        Err(CallbackError::Internal(e)) => Err(e),
        Ok((v,sidecar)) => Ok((cb(v),sidecar))
    }
}

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
            let (res,sidecar) = match attempt.request() {
                MiniRequest::Jump(req) => { 
                    map_error(self.backend.jump(req).await, |r| MiniResponse::Jump(r))?
                },
                MiniRequest::BootChannel(req) => {
                    map_error(self.backend.boot(req).await, |r| MiniResponse::BootChannel(r))?
                },
                MiniRequest::Stick(req) => {
                    map_error(self.backend.stickinfo(req).await,|r| MiniResponse::Stick(r))?
                },
                MiniRequest::Expand(req) => {
                    map_error(self.backend.expansion(req).await,|r| MiniResponse::Expand(r))?
                },
                MiniRequest::Program(req) => {
                    map_error(self.backend.program(req).await,|r| MiniResponse::Program(r))?
                },
                MiniRequest::Data(req) => {
                    map_error(self.backend.data(req).await,|r| MiniResponse::Data(r))?
                },
                _ => { 
                    (MiniResponse::FailureRes(FailureRes::new("unimplemented")),JsSidecar::new_empty())
                }
            };
            out.add_response(attempt.make_response_attempt(res));
            sidecars.merge(sidecar);
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
