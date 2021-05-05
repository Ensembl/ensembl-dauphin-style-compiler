use std::sync::{ Arc };
use crate::agent::agent::Agent;
use crate::util::memoized::{ MemoizedType };
use crate::Scale;
use crate::lane::ShapeRequest;
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

fn map_scale(lane: &ShapeRequest, scale: &Scale) -> u64 {
    lane.index() >> (scale.get_index() - lane.scale().get_index())
}

fn to_candidate(lane: &ShapeRequest, program_region: &ProgramRegion) -> ShapeRequest {
    let scale = program_region.scale_up(&lane.scale());
    let index = map_scale(lane,&scale);
    Lane::new(lane.stick_id().clone(),index, scale,lane.track_config().clone())
}

pub async fn scaler(stick_store: &StickStore, lane_program_lookup: &LaneProgramLookup, lane: &ShapeRequest) -> Result<ShapeRequest,DataMessage> {
    let tags : Vec<String> = stick_store.get(&lane.region().stick()).await?.as_ref().tags().iter().cloned().collect();
    let program_region_query = ProgramRegionQuery::new(&tags,&lane.region().scale(),lane.track().track().program_name());
    let program_region = lane_program_lookup.get(&program_region_query).ok_or_else(|| DataMessage::NoLaneProgram(lane.clone()))?;
    let candidate = lane.best_scale();
    Ok(candidate)
}

async fn scale(base: PeregrineCoreBase, agent_store: AgentStore, lane: ShapeRequest) -> Result<ShapeRequest,DataMessage> {
    let stick_store = agent_store.stick_store().await;
    let lane_program_lookup = agent_store.lane_program_lookup().await;
    handle_missing_data(&base.messages,scaler(&stick_store,&lane_program_lookup,&lane).await)
}

#[derive(Clone)]
pub struct LaneScaler(Agent<ShapeRequest,ShapeRequest>);

impl LaneScaler {
    pub fn new(cache_size: usize, base: &PeregrineCoreBase, agent_store: &AgentStore) -> LaneScaler {
        LaneScaler(Agent::new(MemoizedType::Cache(cache_size),"lane-programs",1,base,agent_store, scale))
    }

    pub async fn get(&self, lane: &ShapeRequest) -> Arc<Result<ShapeRequest,DataMessage>> {
        self.0.get(lane).await
    }
}
