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

mod util;

/* interp */
mod interp {
    mod boot;
    mod data;
    mod geometry;
    mod panel;
    mod shape;
    pub mod library;
}
pub use interp::library::make_peregrine_interp;

mod payloads {
    mod geometrybuilder;
    mod panelbuilder;
    mod payload;
    pub use payload::{ PeregrinePayloadFactory, PeregrinePayload, add_peregrine_payloads };
}

pub use payloads::add_peregrine_payloads;

/* compile */
#[cfg(any(feature = "compile",test))]
mod compile {
    mod boot;
    mod data;
    mod geometry;
    mod panel;
    mod shape;
    pub mod library;
}

#[cfg(any(feature = "compile",test))]
pub use compile::library::make_peregrine;
