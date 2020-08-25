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
use peregrine_core::{ StickAuthorityStore };

pub struct PeregrinePayload {
    sas: StickAuthorityStore
}

impl Payload for PeregrinePayload {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn finish(&mut self) {}
}

impl PeregrinePayload {
    fn new(sas: &StickAuthorityStore) -> PeregrinePayload {
        PeregrinePayload {
            sas: sas.clone()
        }
    }

    pub fn stick_authority_store(&self) -> &StickAuthorityStore { &self.sas }
}

#[derive(Clone)]
pub struct PeregrinePayloadFactory {
    sas: StickAuthorityStore
}

impl PeregrinePayloadFactory {
    pub fn new(sas: &StickAuthorityStore) -> PeregrinePayloadFactory {
        PeregrinePayloadFactory {
            sas: sas.clone()
        }
    }
}

impl PayloadFactory for PeregrinePayloadFactory {
    fn make_payload(&self) -> Box<dyn Payload> {
        Box::new(PeregrinePayload::new(&self.sas))
    }
}

pub fn add_peregrine_payloads(dauphin: &mut Dauphin, sas: &StickAuthorityStore) {
    dauphin.add_payload_factory("peregrine","core",Box::new(PeregrinePayloadFactory::new(sas)))
}
