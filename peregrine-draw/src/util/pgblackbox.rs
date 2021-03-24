//#![cfg(blackbox)] XXX
use std::hash::Hasher;
use std::sync::{ Arc, Mutex };
use blackbox::{ Integration, blackbox_config, blackbox_integration, blackbox_take_json, blackbox_enable, blackbox_log };
use commander::{ cdr_timer };
use js_sys::Date;
use js_sys::Math::random;
use crate::{ PgAjax, Message, PgCommanderWeb };
use serde_json::Value as JsonValue;
use fnv::FnvHasher;
use base64;
use url::Url;
use web_sys::console;

#[derive(Clone)]
pub struct PgBlackboxIntegration {
    instance: String,
    endpoint: Arc<Mutex<Option<Url>>>
}

fn instance_id() -> String {
    let mut  h = FnvHasher::with_key(1);
    h.write_i64((random()*1000000000.) as i64);
    h.write_i64(Date::now() as i64);
    base64::encode(h.finish().to_string())[0..5].to_string()
}

impl PgBlackboxIntegration {
    pub fn new() -> PgBlackboxIntegration {
        PgBlackboxIntegration{
            instance: instance_id().to_string(),
            endpoint: Arc::new(Mutex::new(None))
        }
    }

    async fn send_data(&self, data: &JsonValue) -> Result<(),Message> {
        if let Some(url) = self.url() {
            let mut ajax = PgAjax::new("POST",&url);
            let mut buffer = Vec::new();
            serde_json::to_writer(&mut buffer,&data).map_err(|e| Message::SerializationError(e.to_string()))?;
            ajax.set_body(buffer);
            let response = ajax.get_json().await?;
            blackbox_config(&response);
        }
        Ok(())
    }    

    pub async fn sync_task(&self) -> Result<(),Message> {
        loop {
            let data = blackbox_take_json();
            if let Err(e) = self.send_data(&data).await {
                console::log_1(&format!("blackbox: {}",e).into());
            }
            cdr_timer(10000.).await;
        }
    }
    
    pub fn set_url(&mut self, url: &Url) {
        *self.endpoint.lock().unwrap() = Some(url.clone());
    }

    pub fn url(&self) -> Option<Url> {  self.endpoint.lock().unwrap().as_ref().cloned() }
}

impl Integration for PgBlackboxIntegration {
    fn get_time(&self) -> f64 {
        Date::now()
    }

    fn get_instance_id(&self) -> String { self.instance.clone() }

    fn get_time_units(&self) -> String { "ms".to_string() }
}

pub fn pgblackbox_setup() -> PgBlackboxIntegration {
    let ign = PgBlackboxIntegration::new();
    blackbox_integration(ign.clone());
    ign
}

//#[cfg(blackbox)]
pub fn setup_blackbox(commander: &PgCommanderWeb, url: &str) {
    let mut ign = pgblackbox_setup();
    ign.set_url(&Url::parse(url).expect("bad blackbox url"));
    let ign2 = ign.clone();
    blackbox_enable("notice");
    blackbox_enable("warn");
    blackbox_enable("error");
    commander.add::<()>("blackbox",10,None,None,Box::pin(async move { ign2.sync_task().await; Ok(()) }));
    blackbox_log("general","blackbox configured");
    console::log_1(&format!("blackbox configured").into());
}

/*
#[cfg(not(blackbox))]
pub fn setup_blackbox(_commander: &PgCommanderWeb, url: &str) {
}
*/