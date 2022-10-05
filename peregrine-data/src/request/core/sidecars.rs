use crate::{PgDauphin, run::pgdauphin::add_programs_from_response, ResponsePacket, api::MessageSender, BackendNamespace};

#[derive(Clone)]
pub(crate) struct RequestSidecars {
    pgd: PgDauphin
}

impl RequestSidecars {
    pub(crate) fn new(pgd: &PgDauphin) -> RequestSidecars {
        RequestSidecars {
            pgd: pgd.clone()
        }
    }

    pub(crate) async fn run(&self, response: &ResponsePacket, channel: &BackendNamespace, messages: &MessageSender) {
        add_programs_from_response(&self.pgd,channel,response,messages).await;
    }
}
