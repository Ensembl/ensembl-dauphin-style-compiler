use std::{sync::{Arc, Mutex}, collections::HashMap };
use peregrine_data::{ChannelIntegration, ChannelSender, BackendNamespace };
use peregrine_toolkit::{ lock, error::Error, log };
use wasm_bindgen::JsValue;
use crate::channel::JavascriptChannel;

#[derive(Clone)]
pub struct JavascriptIntegration {
    channels: Arc<Mutex<HashMap<String,Arc<JavascriptChannel>>>>
}

impl JavascriptIntegration {
    pub fn new() -> JavascriptIntegration {
        JavascriptIntegration {
            channels: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    pub fn add_channel(&self, name: &str, payload: JsValue) -> Result<(),Error> {
        lock!(self.channels).insert(name.to_string(),Arc::new(JavascriptChannel::new(name,payload)?));
        Ok(())
    }
}

impl ChannelIntegration for JavascriptIntegration {
    fn make_channel(&self, name: &str) -> Option<(Arc<dyn ChannelSender>,Option<BackendNamespace>)> {
        if let Some((prefix,suffix)) = name.split_once(":") {
            log!("try prefix='{}' suffix='{}'",prefix,suffix);
            if prefix == "jsapi" {
                if let Some(channel) = lock!(self.channels).get(suffix) {
                    return Some((channel.clone(),Some(channel.backend_namespace().clone())));
                }
            }
        }
        None
    }
}
