use std::sync::{ Arc };
use crate::agent::agent::Agent;
use crate::util::memoized::{ MemoizedType };
use crate::Scale;
use crate::lane::Lane;
pub use crate::util::message::DataMessage;
use crate::api::{ PeregrineCoreBase, AgentStore, MessageSender };
use crate::index::StickStore;
use crate::agent::laneprogramlookup::LaneProgramLookup;
use crate::lane::programregion::{ ProgramRegion, ProgramRegionQuery };

fn handle_missing_data(message_sender: &MessageSender, r: Result<Lane,DataMessage>) -> Result<Lane,DataMessage> {
    r.map_err(|e| {
        message_sender.send(e.clone());
        DataMessage::DataMissing(Box::new(e))
    })
}

fn map_scale(lane: &Lane, scale: &Scale) -> u64 {
    lane.index() >> (scale.get_index() - lane.scale().get_index())
}

fn to_candidate(lane: &Lane, program_region: &ProgramRegion) -> Lane {
    let scale = program_region.scale_up(&lane.scale());
    let index = map_scale(lane,&scale);
    Lane::new(lane.stick_id().clone(),index, scale,lane.track_config().clone())
}

pub async fn scaler(stick_store: &StickStore, lane_program_lookup: &LaneProgramLookup, lane: &Lane) -> Result<Lane,DataMessage> {
    let tags : Vec<String> = stick_store.get(&lane.stick_id()).await?.as_ref().tags().iter().cloned().collect();
    let program_region_query = ProgramRegionQuery::new(&tags,&lane.scale(),lane.track_config().track().program_name());
    let program_region = lane_program_lookup.get(&program_region_query).ok_or_else(|| DataMessage::NoLaneProgram(lane.clone()))?;
    let candidate = to_candidate(lane,&program_region);
    Ok(candidate)
}

async fn scale(base: PeregrineCoreBase, agent_store: AgentStore, lane: Lane) -> Result<Lane,DataMessage> {
    let stick_store = agent_store.stick_store().await;
    let lane_program_lookup = agent_store.lane_program_lookup().await;
    handle_missing_data(&base.messages,scaler(&stick_store,&lane_program_lookup,&lane).await)
}

#[derive(Clone)]
pub struct LaneScaler(Agent<Lane,Lane>);

impl LaneScaler {
    pub fn new(cache_size: usize, base: &PeregrineCoreBase, agent_store: &AgentStore) -> LaneScaler {
        LaneScaler(Agent::new(MemoizedType::Cache(cache_size),"lane-programs",1,base,agent_store, scale))
    }

    pub async fn get(&self, lane: &Lane) -> Arc<Result<Lane,DataMessage>> {
        self.0.get(lane).await
    }
}
