use peregrine_toolkit::error::err_web_drop;

use crate::{PgDauphin, run::pgdauphin::add_programs_from_response, MaxiResponse, api::MessageSender, BackendNamespace, request::tracks::trackdata::add_tracks_from_response, PeregrineApiQueue, Switches};

#[derive(Clone)]
pub(crate) struct RequestSidecars {
    pgd: PgDauphin,
    switches: Box<Switches>,
    queue: PeregrineApiQueue
}

impl RequestSidecars {
    pub(crate) fn new(pgd: &PgDauphin, switches: &Switches, queue: &PeregrineApiQueue) -> RequestSidecars {
        RequestSidecars {
            pgd: pgd.clone(),
            queue: queue.clone(),
            switches: Box::new(switches.clone())
        }
    }

    pub(crate) async fn run(&self, response: &MaxiResponse, channel: &BackendNamespace, messages: &MessageSender) {
        err_web_drop(add_programs_from_response(&self.pgd,channel,response,messages).await);
        err_web_drop(add_tracks_from_response(response,&self.switches,&self.queue,&self.pgd).await);
    }
}
