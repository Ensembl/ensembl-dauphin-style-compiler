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
use blackbox::{ blackbox_log };
use dauphin_interp::runtime::{ Payload, PayloadFactory };
use dauphin_interp::{ Dauphin };
use peregrine_core::{ StickAuthorityStore, StickStore, RequestManager, CountingPromise, PanelProgram };
use super::panelbuilder::PanelBuilder;

pub struct PeregrinePayload {
    booted: CountingPromise,
    sas: StickAuthorityStore,
    ss: StickStore,
    manager: RequestManager,
    panel_program: PanelProgram,
    panel_builder: PanelBuilder
}

impl Payload for PeregrinePayload {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn finish(&mut self) {}
}

impl PeregrinePayload {
    fn new(sas: &StickAuthorityStore, ss: &StickStore, manager: &RequestManager, booted: &CountingPromise, panel_program: &PanelProgram) -> PeregrinePayload {
        PeregrinePayload {
            booted: booted.clone(),
            sas: sas.clone(),
            ss: ss.clone(),
            panel_program: panel_program.clone(),
            manager: manager.clone(),
            panel_builder: PanelBuilder::new()
        }
    }

    pub fn stick_authority_store(&self) -> &StickAuthorityStore { &self.sas }
    pub fn stick_store(&self) -> &StickStore { &self.ss }
    pub fn manager(&self) -> &RequestManager { &self.manager }
    pub fn booted(&self) -> &CountingPromise { &self.booted }
    pub fn panel_builder(&self) -> &PanelBuilder { &self.panel_builder }
    pub fn panel_program(&self) -> &PanelProgram { &self.panel_program }
}

#[derive(Clone)]
pub struct PeregrinePayloadFactory {
    manager: RequestManager,
    ss: StickStore,
    sas: StickAuthorityStore,
    pp: PanelProgram,
    booted: CountingPromise
}

impl PeregrinePayloadFactory {
    pub fn new(manager: &RequestManager, ss: &StickStore, sas: &StickAuthorityStore, booted: &CountingPromise, pp: &PanelProgram) -> PeregrinePayloadFactory {
        PeregrinePayloadFactory {
            booted: booted.clone(),
            manager: manager.clone(),
            pp: pp.clone(),
            ss: ss.clone(),
            sas: sas.clone()
        }
    }
}

impl PayloadFactory for PeregrinePayloadFactory {
    fn make_payload(&self) -> Box<dyn Payload> {
        Box::new(PeregrinePayload::new(&self.sas,&self.ss,&self.manager,&self.booted,&self.pp))
    }
}

pub fn add_peregrine_payloads(dauphin: &mut Dauphin, manager: &RequestManager, ss: &StickStore, sas: &StickAuthorityStore, booted: &CountingPromise, panel_program: &PanelProgram) {
    dauphin.add_payload_factory("peregrine","core",Box::new(PeregrinePayloadFactory::new(manager,ss,sas,booted,panel_program)))
}
