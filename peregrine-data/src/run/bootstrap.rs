use std::sync::{Arc, Mutex};
use crate::{AgentStore, DataMessage, PeregrineApiQueue, PeregrineCoreBase, PeregrineIntegration, PgCommanderTaskSpec, PgDauphin, add_task, api::ApiMessage, core::{channel::Channel, version::VersionMetadata}, request::{core::manager::RequestManager, messages::bootstrapres::BootRes}, shapeload::programloader::ProgramLoader};

use super::PgDauphinTaskSpec;

async fn finish_bootstrap(response: &BootRes, manager: &RequestManager, dauphin: &PgDauphin, queue: &PeregrineApiQueue, loader: &ProgramLoader, integration: &Arc<Mutex<Box<dyn PeregrineIntegration>>>, version: &VersionMetadata) -> Result<(),DataMessage> {
    manager.set_supported_versions(response.supports(),version.backend_version());
    manager.set_lo_divert(response.channel_hi(),response.channel_lo());
    dauphin.run_program(loader,PgDauphinTaskSpec {
        prio: 2,
        slot: None,
        timeout: None,
        program_name: response.program_name().clone(),
        payloads: None
    }).await?;
    integration.lock().unwrap().set_assets(response.assets().clone()); // XXX don't clone
    queue.push(ApiMessage::SetAssets(response.assets().clone()));
    queue.push(ApiMessage::RegenerateTrackConfig);
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
            finish_bootstrap(&response,&base2.manager,&base2.dauphin,&base2.queue,&agent_store.program_loader,&base2.integration,&base2.version).await?;
            base2.booted.unlock();
            Ok(())
        }),
        stats: false
    });
}
