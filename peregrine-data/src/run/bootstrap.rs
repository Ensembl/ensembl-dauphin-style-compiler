use std::sync::{Arc, Mutex};
use peregrine_toolkit::{error, lock};
use crate::{AgentStore, DataMessage, PeregrineApiQueue, PeregrineCoreBase, PeregrineIntegration, PgCommanderTaskSpec, PgDauphin, add_task, core::{channel::{Channel}, version::VersionMetadata}, request::{core::manager::RequestManager, messages::bootstrapres::BootRes}, shapeload::programloader::ProgramLoader, InstanceInformation};
use super::PgDauphinTaskSpec;

async fn finish_bootstrap(response: &BootRes, manager: &RequestManager, dauphin: &PgDauphin, queue: &PeregrineApiQueue, loader: &ProgramLoader, integration: &Arc<Mutex<Box<dyn PeregrineIntegration>>>, version: &VersionMetadata, channel: &Channel) -> Result<(),DataMessage> {
    let info = InstanceInformation::new(
        channel,response,version
    );
    lock!(integration).report_instance_information(&info);
    if let Some(channel_lo) = response.channel_lo() {
        manager.set_lo_divert(channel,channel_lo);
    }
    let r = dauphin.run_program(loader,PgDauphinTaskSpec {
        prio: 2,
        slot: None,
        timeout: None,
        program_name: response.program_name().clone(),
        payloads: None
    }).await;
    if let Err(err) = r {
        error!("{}",err);
    }
    integration.lock().unwrap().set_assets(response.assets().clone()); // XXX don't clone
    queue.set_assets(response.assets());
    queue.regenerate_track_config();
    Ok(())
}

pub(crate) fn bootstrap(base: &PeregrineCoreBase, agent_store: &AgentStore, channel: Channel, identity: u64) {
    *base.identity.lock().unwrap() = identity;
    let base2 = base.clone();
    let agent_store = agent_store.clone();
    let backend = base.all_backends.backend(&channel);
    add_task(&base.commander,PgCommanderTaskSpec {
        name: "bootstrap".to_string(),
        prio: 4,
        slot: None,
        timeout: None,
        task: Box::pin(async move {
            let response = backend.bootstrap().await?;
            finish_bootstrap(&response,&base2.manager,&base2.dauphin,&base2.queue,&agent_store.program_loader,&base2.integration,&base2.version,&channel).await?;
            base2.booted.unlock();
            Ok(())
        }),
        stats: false
    });
}
