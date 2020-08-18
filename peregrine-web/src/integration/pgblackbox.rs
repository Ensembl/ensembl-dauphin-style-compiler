use anyhow::{ self, Context };
use std::hash::Hasher;
use std::sync::Mutex;
use blackbox::{ Integration, blackbox_config, blackbox_integration, blackbox_take_json };
use commander::{ cdr_timer };
use js_sys::Date;
use js_sys::Math::random;
use web_sys::{ XmlHttpRequest, XmlHttpRequestResponseType, console };
use lazy_static::lazy_static;
use crate::util::ajax::PgAjax;
use crate::util::error::{ display_error, js_error, js_warn };
use serde_json::Value as JsonValue;
use fnv::FnvHasher;
use base64;
use url::Url;

lazy_static! {
    static ref ENDPOINT: Mutex<Option<Url>> = Mutex::new(None);
}

pub struct PgBlackboxIntegration(String);

impl PgBlackboxIntegration {
    pub fn new(instance_id: &str) -> PgBlackboxIntegration {
        PgBlackboxIntegration(instance_id.to_string())
    }
}

impl Integration for PgBlackboxIntegration {
    fn get_time(&self) -> f64 {
        Date::now()
    }

    fn get_instance_id(&self) -> String { self.0.clone() }

    fn get_time_units(&self) -> String { "ms".to_string() }
}

fn instance_id() -> String {
    let mut  h = FnvHasher::with_key(1);
    h.write_i64((random()*1000000000.) as i64);
    h.write_i64(Date::now() as i64);
    base64::encode(h.finish().to_string())[0..5].to_string()
}

async fn send_data(url: &Url, data: &JsonValue) -> anyhow::Result<()> {
    let mut ajax = PgAjax::new("POST",url);
    let mut buffer = Vec::new();
    display_error(serde_json::to_writer(&mut buffer,&data))?;
    ajax.set_body(buffer);
    let response = ajax.get_json().await.context("sending blackbox data")?;
    blackbox_config(&response);
    Ok(())
}

pub async fn pgblackbox_sync() -> anyhow::Result<()> {
    loop {
        if let Some(url) = &*ENDPOINT.lock().unwrap() {
            let data = blackbox_take_json();
            js_warn(send_data(url,&data).await.context("sending blackbox"));
        }
        cdr_timer(10000.).await;
    }
}

pub fn pgblackbox_setup() {
    let instance_id = instance_id();
    blackbox_integration(PgBlackboxIntegration::new(&instance_id));
}

pub fn pgblackbox_endpoint(endpoint: Option<&Url>) {
    *ENDPOINT.lock().unwrap() = endpoint.cloned();
}
