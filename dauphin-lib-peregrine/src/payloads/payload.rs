/* 
 *  See the NOTICE file distributed with this work for additional information
 *  regarding copyright ownership.
 *  
 *  Licensed under the Apache License, Version 2.0 (the "License"); you may 
 *  not use this file except in compliance with the License. You may obtain a
 *  copy of the License at http://www.apache.org/licenses/LICENSE-2.0
 *  
 *  Unless required by applicable law or agreed to in writing, software
 *  distributed under the License is distributed on an "AS IS" BASIS, WITHOUT 
 *  WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  
 *  See the License for the specific language governing permissions and
 *  limitations under the License.
 */

use std::any::Any;
use dauphin_interp::runtime::{ Payload, PayloadFactory };
use dauphin_interp::{ Dauphin };
use peregrine_data::{ RequestManager, CountingPromise, AgentStore };
use super::lanebuilder::LaneBuilder;
use super::geometrybuilder::GeometryBuilder;

pub struct PeregrinePayload {
    booted: CountingPromise,
    agent_store: AgentStore,
    manager: RequestManager,
    lane_builder: LaneBuilder,
    geometry_builder: GeometryBuilder
}

impl Payload for PeregrinePayload {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn finish(&mut self) {}
}

impl PeregrinePayload {
    fn new(agent_store: &AgentStore, manager: &RequestManager, booted: &CountingPromise) -> PeregrinePayload {
        PeregrinePayload {
            booted: booted.clone(),
            agent_store: agent_store.clone(),
            manager: manager.clone(),
            lane_builder: LaneBuilder::new(),
            geometry_builder: GeometryBuilder::new()
        }
    }

    pub fn agent_store(&self) -> &AgentStore { &self.agent_store }
    pub fn manager(&self) -> &RequestManager { &self.manager }
    pub fn booted(&self) -> &CountingPromise { &self.booted }
    pub fn lane_builder(&self) -> &LaneBuilder { &self.lane_builder }
    pub fn geometry_builder(&self) -> &GeometryBuilder { &self.geometry_builder }
}

#[derive(Clone)]
pub struct PeregrinePayloadFactory {
    manager: RequestManager,
    agent_store: AgentStore,
    booted: CountingPromise
}

impl PeregrinePayloadFactory {
    pub fn new(manager: &RequestManager, agent_store: &AgentStore, booted: &CountingPromise) -> PeregrinePayloadFactory {
        PeregrinePayloadFactory {
            booted: booted.clone(),
            manager: manager.clone(),
            agent_store: agent_store.clone()
        }
    }
}

impl PayloadFactory for PeregrinePayloadFactory {
    fn make_payload(&self) -> Box<dyn Payload> {
        Box::new(PeregrinePayload::new(&self.agent_store,&self.manager,&self.booted))
    }
}

pub fn add_peregrine_payloads(dauphin: &mut Dauphin, manager: &RequestManager,
                                agent_store: &AgentStore, booted: &CountingPromise) {
    dauphin.add_payload_factory("peregrine","core",Box::new(PeregrinePayloadFactory::new(manager,agent_store,booted)))
}
