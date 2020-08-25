use anyhow;
use dauphin_interp::Dauphin;
use crate::integration::stream::WebStreamFactory;
use peregrine_dauphin::PgDauphinIntegration;

pub struct PgDauphinIntegrationWeb();

impl PgDauphinIntegration for PgDauphinIntegrationWeb {
    fn add_payloads(&self, dauphin: &mut Dauphin) {
        dauphin.add_payload_factory("std","stream",Box::new(WebStreamFactory::new()));
    }
}
