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
use peregrine_data::{AgentStore, CountingPromise, PeregrineCoreBase };
use super::geometrybuilder::GeometryBuilder;

pub struct PeregrinePayload {
    booted: CountingPromise,
    agent_store: AgentStore,
    geometry_builder: GeometryBuilder
}

impl Payload for PeregrinePayload {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn finish(&mut self) {}
}

impl PeregrinePayload {
    fn new(agent_store: &AgentStore, booted: &CountingPromise) -> PeregrinePayload {
        PeregrinePayload {
            booted: booted.clone(),
            agent_store: agent_store.clone(),
            geometry_builder: GeometryBuilder::new(),
        }
    }

    pub fn agent_store(&self) -> &AgentStore { &self.agent_store }
    pub fn booted(&self) -> &CountingPromise { &self.booted }
    pub fn geometry_builder(&self) -> &GeometryBuilder { &self.geometry_builder }
}

#[derive(Clone)]
pub struct PeregrinePayloadFactory {
    agent_store: AgentStore,
    booted: CountingPromise
}

impl PeregrinePayloadFactory {
    pub fn new(base: &PeregrineCoreBase, agent_store: &AgentStore) -> PeregrinePayloadFactory {
        PeregrinePayloadFactory {
            booted: base.booted.clone(),
            agent_store: agent_store.clone(),
        }
    }
}

impl PayloadFactory for PeregrinePayloadFactory {
    fn make_payload(&self) -> Box<dyn Payload> {
        Box::new(PeregrinePayload::new(&self.agent_store,&self.booted))
    }
}

pub fn add_peregrine_payloads(dauphin: &mut Dauphin, base: &PeregrineCoreBase,agent_store: &AgentStore) {
    dauphin.add_payload_factory("peregrine","core",Box::new(PeregrinePayloadFactory::new(&base,agent_store)));
}
