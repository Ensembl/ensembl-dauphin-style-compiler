use crate::{PgDauphin, run::pgdauphin::add_programs_from_response, ResponsePacket, Channel, api::MessageSender};

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

    pub(crate) async fn run(&self, response: &ResponsePacket, channel: &Channel, messages: &MessageSender) {
        add_programs_from_response(&self.pgd,channel,response,messages).await;
    }
}
