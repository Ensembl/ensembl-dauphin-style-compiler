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
use peregrine_data::{AgentStore, AllotmentMetadataStore, CountingPromise, PeregrineCoreBase, RequestManager, Switches, Universe};
use super::trackbuilder::AllTracksBuilder;
use super::geometrybuilder::GeometryBuilder;

pub struct PeregrinePayload {
    booted: CountingPromise,
    agent_store: AgentStore,
    manager: RequestManager,
    track_builder: AllTracksBuilder,
    geometry_builder: GeometryBuilder,
    allotment_metadata: AllotmentMetadataStore,
    switches: Switches
}

impl Payload for PeregrinePayload {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn finish(&mut self) {}
}

impl PeregrinePayload {
    fn new(agent_store: &AgentStore, manager: &RequestManager, booted: &CountingPromise, switches: &Switches, allotment_metadata: &AllotmentMetadataStore) -> PeregrinePayload {
        PeregrinePayload {
            booted: booted.clone(),
            agent_store: agent_store.clone(),
            manager: manager.clone(),
            track_builder: AllTracksBuilder::new(),
            geometry_builder: GeometryBuilder::new(),
            switches: switches.clone(),
            allotment_metadata: allotment_metadata.clone(),
        }
    }

    pub fn switches(&self) -> &Switches { &self.switches }
    pub fn agent_store(&self) -> &AgentStore { &self.agent_store }
    pub fn manager(&self) -> &RequestManager { &self.manager }
    pub fn booted(&self) -> &CountingPromise { &self.booted }
    pub fn track_builder(&self) -> &AllTracksBuilder { &self.track_builder }
    pub fn geometry_builder(&self) -> &GeometryBuilder { &self.geometry_builder }
    pub fn allotment_metadata(&self) -> &AllotmentMetadataStore { &self.allotment_metadata }
}

#[derive(Clone)]
pub struct PeregrinePayloadFactory {
    manager: RequestManager,
    agent_store: AgentStore,
    booted: CountingPromise,
    switches: Switches,
    allotment_metadata: AllotmentMetadataStore,
}

impl PeregrinePayloadFactory {
    pub fn new(base: &PeregrineCoreBase, agent_store: &AgentStore, switches: &Switches) -> PeregrinePayloadFactory {
        PeregrinePayloadFactory {
            booted: base.booted.clone(),
            manager: base.manager.clone(),
            agent_store: agent_store.clone(),
            switches: switches.clone(),
            allotment_metadata: base.allotment_metadata.clone()
        }
    }
}

impl PayloadFactory for PeregrinePayloadFactory {
    fn make_payload(&self) -> Box<dyn Payload> {
        Box::new(PeregrinePayload::new(&self.agent_store,&self.manager,&self.booted,&self.switches,&self.allotment_metadata))
    }
}

pub fn add_peregrine_payloads(dauphin: &mut Dauphin, base: &PeregrineCoreBase,agent_store: &AgentStore, switches: &Switches) {
    dauphin.add_payload_factory("peregrine","core",Box::new(PeregrinePayloadFactory::new(&base,agent_store,&switches)));
}
